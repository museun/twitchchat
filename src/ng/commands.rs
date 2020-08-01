#![cfg_attr(debug_assertions, allow(dead_code, unused_variables))]
use crate::ng::Encodable;
use std::{
    convert::TryInto,
    io::{Result, Write},
};

struct ByteWriter<'a, W: Write + ?Sized>(&'a mut W);
impl<'a, W: Write + ?Sized> ByteWriter<'a, W> {
    fn new(writer: &'a mut W) -> Self {
        Self(writer)
    }

    fn write_bytes(self, data: impl AsRef<[u8]>) -> Result<()> {
        self.0.write_all(data.as_ref())?;
        self.endline()
    }

    fn endline(self) -> Result<()> {
        self.0.write_all(b"\r\n")
    }

    fn parts_term(self, parts: &[&dyn AsRef<[u8]>]) -> Result<()> {
        parts
            .iter()
            .map(|p| self.0.write_all(p.as_ref()))
            .collect::<Result<()>>()?;
        self.endline()
    }

    fn parts(self, parts: &[&dyn AsRef<[u8]>]) -> Result<()> {
        parts
            .iter()
            .filter_map(|p| {
                let part = p.as_ref();
                if part.is_empty() {
                    None
                } else {
                    Some(part)
                }
            })
            .enumerate()
            .map(|(i, part)| {
                if i > 0 {
                    self.0.write_all(b" ")?;
                }
                self.0.write_all(part.as_ref())
            })
            .collect::<Result<()>>()?;
        self.endline()
    }

    fn jtv_command(self, parts: &[&dyn AsRef<[u8]>]) -> Result<()> {
        self.0.write_all(b"PRIVMSG jtv :")?;
        self.parts(parts)
    }

    fn command(self, channel: impl AsRef<[u8]>, parts: &[&dyn AsRef<[u8]>]) -> Result<()> {
        self.0.write_all(b"PRIVMSG ")?;
        self.0.write_all(channel.as_ref())?;
        self.0.write_all(b" :")?;
        self.parts(parts)
    }
}

#[cfg(feature = "serde")]
struct SerdeWrapper<'a, T: 'a>(T, &'static str, std::marker::PhantomData<&'a T>);

#[cfg(feature = "serde")]
impl<'a, T: 'a> ::serde::Serialize for SerdeWrapper<'a, T>
where
    T: Encodable,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        use ::serde::ser::{Error, SerializeStruct as _};

        let Self(item, name, ..) = self;

        let mut data = vec![];
        item.encode(&mut data).map_err(Error::custom)?;
        let raw = std::str::from_utf8(&data).map_err(Error::custom)?;

        let mut s = serializer.serialize_struct(name, 1)?;
        s.serialize_field("raw", raw)?;
        s.end()
    }
}

macro_rules! serialize {
    ($($ty:ident)*) => {
        $(
            #[cfg(feature = "serde")]
            impl<'a> ::serde::Serialize for $ty<'a> {
                fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
                where
                    S: ::serde::Serializer,
                {
                    SerdeWrapper(self, stringify!($ty), std::marker::PhantomData).serialize(serializer)
                }
            }
        )*
    };
}

serialize! {
    Ban
    Clear
    Color
    Command
    JtvCommand
    Commercial
    Disconnect
    EmoteOnly
    EmoteOnlyOff
    Followers
    FollowersOff
    GiveMod
    Help
    Host
    Join
    Marker
    Me
    Mods
    Part
    Ping
    Pong
    Privmsg
    R9kBeta
    R9kBetaOff
    Raid
    Raw
    Slow
    SlowOff
    Subscribers
    SubscribersOff
    Timeout
    Unban
    Unhost
    Unmod
    Unraid
    Untimeout
    Unvip
    Vip
    Vips
    Whisper
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Ban<'a> {
    channel: &'a str,
    username: &'a str,
    reason: Option<&'a str>,
}

impl<'a> Encodable for Ban<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(
            self.channel,
            &[&"/ban", &self.username, &self.reason.unwrap_or_default()],
        )
    }
}

pub fn ban<'a>(channel: &'a str, username: &'a str, reason: impl Into<Option<&'a str>>) -> Ban<'a> {
    Ban {
        channel,
        username,
        reason: reason.into(),
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Clear<'a> {
    channel: &'a str,
}

pub fn clear(channel: &str) -> Clear<'_> {
    Clear { channel }
}

impl<'a> Encodable for Clear<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&self.channel, &[&"/clear"])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Color<'a> {
    color: crate::color::Color,
    marker: std::marker::PhantomData<&'a ()>,
}

pub fn color<T>(color: T) -> std::result::Result<Color<'static>, T::Error>
where
    T: TryInto<crate::color::Color>,
{
    color.try_into().map(|color| Color {
        color,
        marker: std::marker::PhantomData,
    })
}

impl<'a> Encodable for Color<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).jtv_command(&[&"/color", &self.color.to_string()])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Command<'a> {
    channel: &'a str,
    data: &'a str,
}

pub fn command<'a>(channel: &'a str, data: &'a str) -> Command<'a> {
    Command { data, channel }
}

impl<'a> Encodable for Command<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&self.data])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct JtvCommand<'a> {
    data: &'a str,
}

pub fn jtv_command(data: &str) -> JtvCommand<'_> {
    JtvCommand { data }
}

impl<'a> Encodable for JtvCommand<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).jtv_command(&[&self.data])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Commercial<'a> {
    channel: &'a str,
    length: Option<usize>,
}

pub fn commercial(channel: &str, length: impl Into<Option<usize>>) -> Commercial<'_> {
    Commercial {
        channel,
        length: length.into(),
    }
}

impl<'a> Encodable for Commercial<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(
            &self.channel,
            &[
                &"/commercial",
                &self
                    .length
                    .map(|s| s.to_string())
                    .as_deref()
                    .unwrap_or_default(),
            ],
        )
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Disconnect<'a>(std::marker::PhantomData<&'a ()>);

pub fn disconnect() -> Disconnect<'static> {
    Disconnect(std::marker::PhantomData)
}

impl<'a> Encodable for Disconnect<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).jtv_command(&[&"/disconnect"])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct EmoteOnly<'a> {
    channel: &'a str,
}

pub fn emote_only(channel: &str) -> EmoteOnly<'_> {
    EmoteOnly { channel }
}

impl<'a> Encodable for EmoteOnly<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/emoteonly"])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct EmoteOnlyOff<'a> {
    channel: &'a str,
}

pub fn emote_only_off(channel: &str) -> EmoteOnlyOff<'_> {
    EmoteOnlyOff { channel }
}

impl<'a> Encodable for EmoteOnlyOff<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/emoteonlyoff"])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Followers<'a> {
    channel: &'a str,
    duration: &'a str,
}

pub fn followers<'a>(channel: &'a str, duration: &'a str) -> Followers<'a> {
    Followers { channel, duration }
}

impl<'a> Encodable for Followers<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/followers", &self.duration])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct FollowersOff<'a> {
    channel: &'a str,
}

pub fn followers_off(channel: &str) -> FollowersOff<'_> {
    FollowersOff { channel }
}

impl<'a> Encodable for FollowersOff<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/followersoff"])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct GiveMod<'a> {
    channel: &'a str,
    username: &'a str,
}

pub fn give_mod<'a>(channel: &'a str, username: &'a str) -> GiveMod<'a> {
    GiveMod { channel, username }
}

impl<'a> Encodable for GiveMod<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/mod", &self.username])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Help<'a> {
    channel: &'a str,
}

pub fn help(channel: &str) -> Help<'_> {
    Help { channel }
}

impl<'a> Encodable for Help<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/help"])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Host<'a> {
    source: &'a str,
    target: &'a str,
}

pub fn host<'a>(source: &'a str, target: &'a str) -> Host<'a> {
    Host { source, target }
}

impl<'a> Encodable for Host<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.source, &[&"/host", &self.target])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Join<'a> {
    channel: &'a str,
}

pub fn join(channel: &str) -> Join<'_> {
    Join { channel }
}

impl<'a> Encodable for Join<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).parts(&[&"JOIN", &self.channel])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Marker<'a> {
    channel: &'a str,
    comment: Option<&'a str>,
}

pub fn marker<'a>(channel: &'a str, comment: impl Into<Option<&'a str>>) -> Marker<'_> {
    Marker {
        channel,
        comment: comment.into(),
    }
}

impl<'a> Encodable for Marker<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        fn truncate(s: &str) -> &str {
            const MAX: usize = 140;
            if s.len() <= MAX {
                return s;
            }

            for n in (0..=MAX).rev() {
                if s.is_char_boundary(n) {
                    return &s[..n];
                }
            }

            ""
        }

        ByteWriter::new(buf).command(
            self.channel,
            &[&"/marker", &self.comment.map(truncate).unwrap_or_default()],
        )
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Me<'a> {
    channel: &'a str,
    msg: &'a str,
}

pub fn me<'a>(channel: &'a str, msg: &'a str) -> Me<'a> {
    Me { channel, msg }
}

impl<'a> Encodable for Me<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/me", &self.msg])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Mods<'a> {
    channel: &'a str,
}

pub fn mods(channel: &str) -> Mods<'_> {
    Mods { channel }
}

impl<'a> Encodable for Mods<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/mods"])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Part<'a> {
    channel: &'a str,
}

pub fn part(channel: &str) -> Part<'_> {
    Part { channel }
}

impl<'a> Encodable for Part<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).parts(&[&"PART", &self.channel])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Ping<'a> {
    token: &'a str,
}

pub fn ping(token: &str) -> Ping<'_> {
    Ping { token }
}

impl<'a> Encodable for Ping<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).parts(&[&"PING", &self.token])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Pong<'a> {
    token: &'a str,
}

pub fn pong(token: &str) -> Pong<'_> {
    Pong { token }
}

impl<'a> Encodable for Pong<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).parts_term(&[&"PONG", &" :", &self.token])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Privmsg<'a> {
    channel: &'a str,
    data: &'a str,
}

pub fn privmsg<'a>(channel: &'a str, data: &'a str) -> Privmsg<'a> {
    Privmsg { channel, data }
}

impl<'a> Encodable for Privmsg<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).parts_term(&[&"PRIVMSG ", &self.channel, &" :", &self.data])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct R9kBeta<'a> {
    channel: &'a str,
}

pub fn r9k_beta(channel: &str) -> R9kBeta<'_> {
    R9kBeta { channel }
}

impl<'a> Encodable for R9kBeta<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/r9kbeta"])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct R9kBetaOff<'a> {
    channel: &'a str,
}

pub fn r9k_beta_off(channel: &str) -> R9kBetaOff<'_> {
    R9kBetaOff { channel }
}

impl<'a> Encodable for R9kBetaOff<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/r9kbetaoff"])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Raid<'a> {
    source: &'a str,
    target: &'a str,
}

pub fn raid<'a>(source: &'a str, target: &'a str) -> Raid<'a> {
    Raid { source, target }
}

impl<'a> Encodable for Raid<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.source, &[&"/raid", &self.target])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Raw<'a> {
    data: &'a str,
}

pub fn raw(data: &str) -> Raw<'_> {
    Raw { data }
}

impl<'a> Encodable for Raw<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).write_bytes(self.data)
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Slow<'a> {
    channel: &'a str,
    duration: usize,
}

pub fn slow(channel: &str, duration: impl Into<Option<usize>>) -> Slow<'_> {
    Slow {
        channel,
        duration: duration.into().unwrap_or(120),
    }
}

impl<'a> Encodable for Slow<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/slow", &self.duration.to_string()])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct SlowOff<'a> {
    channel: &'a str,
}

pub fn slow_off(channel: &str) -> SlowOff<'_> {
    SlowOff { channel }
}

impl<'a> Encodable for SlowOff<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/slowoff"])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Subscribers<'a> {
    channel: &'a str,
}

pub fn subscribers(channel: &str) -> Subscribers<'_> {
    Subscribers { channel }
}

impl<'a> Encodable for Subscribers<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/subscribers"])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct SubscribersOff<'a> {
    channel: &'a str,
}

pub fn subscribers_off(channel: &str) -> SubscribersOff<'_> {
    SubscribersOff { channel }
}

impl<'a> Encodable for SubscribersOff<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/subscribersoff"])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Timeout<'a> {
    channel: &'a str,
    username: &'a str,
    duration: Option<&'a str>,
    reason: Option<&'a str>,
}

pub fn timeout<'a>(
    channel: &'a str,
    username: &'a str,
    duration: impl Into<Option<&'a str>>,
    reason: impl Into<Option<&'a str>>,
) -> Timeout<'a> {
    Timeout {
        channel,
        username,
        duration: duration.into(),
        reason: reason.into(),
    }
}

impl<'a> Encodable for Timeout<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(
            self.channel,
            &[
                &"/timeout",
                &self.username,
                &self.duration.unwrap_or_default(),
                &self.reason.unwrap_or_default(),
            ],
        )
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Unban<'a> {
    channel: &'a str,
    username: &'a str,
}

pub fn unban<'a>(channel: &'a str, username: &'a str) -> Unban<'a> {
    Unban { channel, username }
}

impl<'a> Encodable for Unban<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/unban", &self.username])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Unhost<'a> {
    channel: &'a str,
}

pub fn unhost(channel: &str) -> Unhost<'_> {
    Unhost { channel }
}

impl<'a> Encodable for Unhost<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/unhost"])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Unmod<'a> {
    channel: &'a str,
    username: &'a str,
}

pub fn unmod<'a>(channel: &'a str, username: &'a str) -> Unmod<'a> {
    Unmod { channel, username }
}

impl<'a> Encodable for Unmod<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/unmod", &self.username])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Unraid<'a> {
    channel: &'a str,
}

pub fn unraid(channel: &str) -> Unraid<'_> {
    Unraid { channel }
}

impl<'a> Encodable for Unraid<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/unraid"])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Untimeout<'a> {
    channel: &'a str,
    username: &'a str,
}

pub fn untimeout<'a>(channel: &'a str, username: &'a str) -> Untimeout<'a> {
    Untimeout { channel, username }
}

impl<'a> Encodable for Untimeout<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&self.channel, &[&"/untimeout", &self.username])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Unvip<'a> {
    channel: &'a str,
    username: &'a str,
}

pub fn unvip<'a>(channel: &'a str, username: &'a str) -> Unvip<'a> {
    Unvip { channel, username }
}

impl<'a> Encodable for Unvip<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/unvip", &self.username])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Vip<'a> {
    channel: &'a str,
    username: &'a str,
}

pub fn vip<'a>(channel: &'a str, username: &'a str) -> Vip<'a> {
    Vip { channel, username }
}

impl<'a> Encodable for Vip<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/vip", &self.username])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Vips<'a> {
    channel: &'a str,
}

pub fn vips(channel: &str) -> Vips<'_> {
    Vips { channel }
}

impl<'a> Encodable for Vips<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(self.channel, &[&"/vips"])
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Whisper<'a> {
    username: &'a str,
    message: &'a str,
}

pub fn whisper<'a>(username: &'a str, message: &'a str) -> Whisper<'a> {
    Whisper { username, message }
}

impl<'a> Encodable for Whisper<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).jtv_command(&[&"/w", &self.username, &self.message])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_encode(
        enc: impl Encodable,
        expected: impl for<'a> PartialEq<&'a str> + std::fmt::Debug,
    ) {
        let mut data = vec![];
        enc.encode(&mut data).unwrap();
        assert_eq!(expected, std::str::from_utf8(&data).unwrap());
    }

    #[test]
    fn encode_raw() {
        test_encode(
            raw("PRIVMSG #test :this is a test"),
            "PRIVMSG #test :this is a test\r\n",
        );
    }

    #[test]
    fn encode_pong() {
        test_encode(pong("123456789"), "PONG :123456789\r\n");
    }

    #[test]
    fn encode_ping() {
        test_encode(ping("123456789"), "PING 123456789\r\n");
    }

    #[test]
    fn encode_join() {
        test_encode(join("#museun"), "JOIN #museun\r\n");
    }

    #[test]
    fn encode_part() {
        test_encode(part("#museun"), "PART #museun\r\n");
    }

    #[test]
    fn encode_privmsg() {
        test_encode(
            privmsg("#museun", "this is a test of a line"),
            "PRIVMSG #museun :this is a test of a line\r\n",
        );

        test_encode(
            privmsg("#museun", &"foo ".repeat(500)),
            format!("PRIVMSG #museun :{}\r\n", &"foo ".repeat(500)),
        );
    }

    #[test]
    fn encode_ban() {
        test_encode(
            ban("#museun", "museun", None),
            "PRIVMSG #museun :/ban museun\r\n",
        )
    }

    #[test]
    fn encode_clear() {
        test_encode(clear("#museun"), "PRIVMSG #museun :/clear\r\n")
    }

    #[test]
    fn encode_color() {
        let blue: crate::color::Color = "blue".parse().unwrap();
        test_encode(
            color(blue).unwrap(),
            format!("PRIVMSG jtv :/color {}\r\n", blue),
        )
    }

    #[test]
    fn encode_command() {
        test_encode(
            command("#museun", "/testing"),
            "PRIVMSG #museun :/testing\r\n",
        )
    }

    #[test]
    fn encode_commercial() {
        test_encode(
            commercial("#museun", None),
            "PRIVMSG #museun :/commercial\r\n",
        );
        test_encode(
            commercial("#museun", 10),
            "PRIVMSG #museun :/commercial 10\r\n",
        );
        test_encode(
            commercial("#museun", Some(10)),
            "PRIVMSG #museun :/commercial 10\r\n",
        );
    }

    #[test]
    fn encode_disconnect() {
        test_encode(disconnect(), "PRIVMSG jtv :/disconnect\r\n")
    }

    #[test]
    fn encode_emoteonly() {
        test_encode(emote_only("#museun"), "PRIVMSG #museun :/emoteonly\r\n")
    }

    #[test]
    fn encode_emoteonlyoff() {
        test_encode(
            emote_only_off("#museun"),
            "PRIVMSG #museun :/emoteonlyoff\r\n",
        )
    }

    #[test]
    fn encode_followers() {
        test_encode(
            followers("#museun", "1 week"),
            "PRIVMSG #museun :/followers 1 week\r\n",
        )
    }

    #[test]
    fn encode_followersoff() {
        test_encode(
            followers_off("#museun"),
            "PRIVMSG #museun :/followersoff\r\n",
        )
    }

    #[test]
    fn encode_help() {
        test_encode(help("#museun"), "PRIVMSG #museun :/help\r\n")
    }

    #[test]
    fn encode_host() {
        test_encode(
            host("#museun", "#shaken_bot"),
            "PRIVMSG #museun :/host #shaken_bot\r\n",
        )
    }

    #[test]
    fn encode_marker() {
        test_encode(
            marker("#museun", Some("this is an example")),
            "PRIVMSG #museun :/marker this is an example\r\n",
        );
        test_encode(
            marker("#museun", "this is an example"),
            "PRIVMSG #museun :/marker this is an example\r\n",
        );
        test_encode(
            marker("#museun", "a".repeat(200).as_str()),
            format!("PRIVMSG #museun :/marker {}\r\n", "a".repeat(140)),
        );
        test_encode(marker("#museun", None), "PRIVMSG #museun :/marker\r\n");
    }

    #[test]
    fn encode_me() {
        test_encode(
            me("#museun", "some emote"),
            "PRIVMSG #museun :/me some emote\r\n",
        );
    }

    #[test]
    fn encode_give_mod() {
        test_encode(
            give_mod("#museun", "shaken_bot"),
            "PRIVMSG #museun :/mod shaken_bot\r\n",
        )
    }

    #[test]
    fn encode_mods() {
        test_encode(mods("#museun"), "PRIVMSG #museun :/mods\r\n")
    }

    #[test]
    fn encode_r9kbeta() {
        test_encode(r9k_beta("#museun"), "PRIVMSG #museun :/r9kbeta\r\n")
    }

    #[test]
    fn encode_r9kbetaoff() {
        test_encode(r9k_beta_off("#museun"), "PRIVMSG #museun :/r9kbetaoff\r\n")
    }

    #[test]
    fn encode_raid() {
        test_encode(
            raid("#museun", "#museun"),
            "PRIVMSG #museun :/raid #museun\r\n",
        )
    }

    #[test]
    fn encode_slow() {
        test_encode(slow("#museun", Some(42)), "PRIVMSG #museun :/slow 42\r\n");
        test_encode(slow("#museun", 42), "PRIVMSG #museun :/slow 42\r\n");
        test_encode(slow("#museun", None), "PRIVMSG #museun :/slow 120\r\n");
    }

    #[test]
    fn encode_slowoff() {
        test_encode(slow_off("#museun"), "PRIVMSG #museun :/slowoff\r\n")
    }

    #[test]
    fn encode_subscribers() {
        test_encode(subscribers("#museun"), "PRIVMSG #museun :/subscribers\r\n")
    }

    #[test]
    fn encode_subscribersoff() {
        test_encode(
            subscribers_off("#museun"),
            "PRIVMSG #museun :/subscribersoff\r\n",
        )
    }

    #[test]
    fn encode_timeout() {
        test_encode(
            timeout("#museun", "museun", None, None),
            "PRIVMSG #museun :/timeout museun\r\n",
        );
        test_encode(
            timeout("#museun", "museun", Some("1d2h"), None),
            "PRIVMSG #museun :/timeout museun 1d2h\r\n",
        );
        test_encode(
            timeout("#museun", "museun", None, Some("spamming")),
            "PRIVMSG #museun :/timeout museun spamming\r\n",
        );
        test_encode(
            timeout("#museun", "museun", Some("1d2h"), Some("spamming")),
            "PRIVMSG #museun :/timeout museun 1d2h spamming\r\n",
        );
    }

    #[test]
    fn encode_unban() {
        test_encode(
            unban("#museun", "museun"),
            "PRIVMSG #museun :/unban museun\r\n",
        )
    }

    #[test]
    fn encode_unhost() {
        test_encode(unhost("#museun"), "PRIVMSG #museun :/unhost\r\n")
    }

    #[test]
    fn encode_unmod() {
        test_encode(
            unmod("#museun", "museun"),
            "PRIVMSG #museun :/unmod museun\r\n",
        )
    }

    #[test]
    fn encode_unraid() {
        test_encode(unraid("#museun"), "PRIVMSG #museun :/unraid\r\n")
    }

    #[test]
    fn encode_untimeout() {
        test_encode(
            untimeout("#museun", "museun"),
            "PRIVMSG #museun :/untimeout museun\r\n",
        )
    }

    #[test]
    fn encode_unvip() {
        test_encode(
            unvip("#museun", "museun"),
            "PRIVMSG #museun :/unvip museun\r\n",
        )
    }

    #[test]
    fn encode_vip() {
        test_encode(vip("#museun", "museun"), "PRIVMSG #museun :/vip museun\r\n")
    }

    #[test]
    fn encode_vips() {
        test_encode(vips("#museun"), "PRIVMSG #museun :/vips\r\n")
    }

    #[test]
    fn encode_whisper() {
        test_encode(
            whisper("museun", "hello world"),
            "PRIVMSG jtv :/w museun hello world\r\n",
        )
    }

    #[cfg(feature = "serde")]
    fn test_serialize(
        enc: impl Encodable + ::serde::Serialize,
        expected: impl for<'a> PartialEq<&'a str> + std::fmt::Debug,
    ) {
        let json = serde_json::to_string_pretty(&enc).unwrap();

        #[derive(Debug, PartialEq, ::serde::Deserialize)]
        struct Wrapper {
            raw: String,
        }

        let wrapper: Wrapper = serde_json::from_str(&json).unwrap();
        assert_eq!(expected, &*wrapper.raw);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_raw() {
        test_serialize(
            raw("PRIVMSG #test :this is a test"),
            "PRIVMSG #test :this is a test\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_pong() {
        test_serialize(pong("123456789"), "PONG :123456789\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_ping() {
        test_serialize(ping("123456789"), "PING 123456789\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_join() {
        test_serialize(join("#museun"), "JOIN #museun\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_part() {
        test_serialize(part("#museun"), "PART #museun\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_privmsg() {
        test_serialize(
            privmsg("#museun", "this is a test of a line"),
            "PRIVMSG #museun :this is a test of a line\r\n",
        );

        test_serialize(
            privmsg("#museun", &"foo ".repeat(500)),
            format!("PRIVMSG #museun :{}\r\n", &"foo ".repeat(500)),
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_ban() {
        test_serialize(
            ban("#museun", "museun", None),
            "PRIVMSG #museun :/ban museun\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_clear() {
        test_serialize(clear("#museun"), "PRIVMSG #museun :/clear\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_color() {
        let blue: crate::color::Color = "blue".parse().unwrap();
        test_serialize(
            color(blue).unwrap(),
            format!("PRIVMSG jtv :/color {}\r\n", blue),
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_command() {
        test_serialize(
            command("#museun", "/testing"),
            "PRIVMSG #museun :/testing\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_commercial() {
        test_serialize(
            commercial("#museun", None),
            "PRIVMSG #museun :/commercial\r\n",
        );
        test_serialize(
            commercial("#museun", 10),
            "PRIVMSG #museun :/commercial 10\r\n",
        );
        test_serialize(
            commercial("#museun", Some(10)),
            "PRIVMSG #museun :/commercial 10\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_disconnect() {
        test_serialize(disconnect(), "PRIVMSG jtv :/disconnect\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_emoteonly() {
        test_serialize(emote_only("#museun"), "PRIVMSG #museun :/emoteonly\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_emoteonlyoff() {
        test_serialize(
            emote_only_off("#museun"),
            "PRIVMSG #museun :/emoteonlyoff\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_followers() {
        test_serialize(
            followers("#museun", "1 week"),
            "PRIVMSG #museun :/followers 1 week\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_followersoff() {
        test_serialize(
            followers_off("#museun"),
            "PRIVMSG #museun :/followersoff\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_help() {
        test_serialize(help("#museun"), "PRIVMSG #museun :/help\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_host() {
        test_serialize(
            host("#museun", "#shaken_bot"),
            "PRIVMSG #museun :/host #shaken_bot\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_marker() {
        test_serialize(
            marker("#museun", Some("this is an example")),
            "PRIVMSG #museun :/marker this is an example\r\n",
        );
        test_serialize(
            marker("#museun", "this is an example"),
            "PRIVMSG #museun :/marker this is an example\r\n",
        );
        test_serialize(
            marker("#museun", "a".repeat(200).as_str()),
            format!("PRIVMSG #museun :/marker {}\r\n", "a".repeat(140)),
        );
        test_serialize(marker("#museun", None), "PRIVMSG #museun :/marker\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_me() {
        test_serialize(
            me("#museun", "some emote"),
            "PRIVMSG #museun :/me some emote\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_give_mod() {
        test_serialize(
            give_mod("#museun", "shaken_bot"),
            "PRIVMSG #museun :/mod shaken_bot\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_mods() {
        test_serialize(mods("#museun"), "PRIVMSG #museun :/mods\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_r9kbeta() {
        test_serialize(r9k_beta("#museun"), "PRIVMSG #museun :/r9kbeta\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_r9kbetaoff() {
        test_serialize(r9k_beta_off("#museun"), "PRIVMSG #museun :/r9kbetaoff\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_raid() {
        test_serialize(
            raid("#museun", "#museun"),
            "PRIVMSG #museun :/raid #museun\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_slow() {
        test_serialize(slow("#museun", Some(42)), "PRIVMSG #museun :/slow 42\r\n");
        test_serialize(slow("#museun", 42), "PRIVMSG #museun :/slow 42\r\n");
        test_serialize(slow("#museun", None), "PRIVMSG #museun :/slow 120\r\n");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_slowoff() {
        test_serialize(slow_off("#museun"), "PRIVMSG #museun :/slowoff\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_subscribers() {
        test_serialize(subscribers("#museun"), "PRIVMSG #museun :/subscribers\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_subscribersoff() {
        test_serialize(
            subscribers_off("#museun"),
            "PRIVMSG #museun :/subscribersoff\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_timeout() {
        test_serialize(
            timeout("#museun", "museun", None, None),
            "PRIVMSG #museun :/timeout museun\r\n",
        );
        test_serialize(
            timeout("#museun", "museun", Some("1d2h"), None),
            "PRIVMSG #museun :/timeout museun 1d2h\r\n",
        );
        test_serialize(
            timeout("#museun", "museun", None, Some("spamming")),
            "PRIVMSG #museun :/timeout museun spamming\r\n",
        );
        test_serialize(
            timeout("#museun", "museun", Some("1d2h"), Some("spamming")),
            "PRIVMSG #museun :/timeout museun 1d2h spamming\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_unban() {
        test_serialize(
            unban("#museun", "museun"),
            "PRIVMSG #museun :/unban museun\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_unhost() {
        test_serialize(unhost("#museun"), "PRIVMSG #museun :/unhost\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_unmod() {
        test_serialize(
            unmod("#museun", "museun"),
            "PRIVMSG #museun :/unmod museun\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_unraid() {
        test_serialize(unraid("#museun"), "PRIVMSG #museun :/unraid\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_untimeout() {
        test_serialize(
            untimeout("#museun", "museun"),
            "PRIVMSG #museun :/untimeout museun\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_unvip() {
        test_serialize(
            unvip("#museun", "museun"),
            "PRIVMSG #museun :/unvip museun\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_vip() {
        test_serialize(vip("#museun", "museun"), "PRIVMSG #museun :/vip museun\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_vips() {
        test_serialize(vips("#museun"), "PRIVMSG #museun :/vips\r\n")
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_whisper() {
        test_serialize(
            whisper("museun", "hello world"),
            "PRIVMSG jtv :/w museun hello world\r\n",
        )
    }
}
