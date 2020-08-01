use std::io::{Result, Write};

pub mod types {
    pub use super::ban::Ban;
    pub use super::clear::Clear;
    pub use super::color::Color;
    pub use super::command::Command;
    pub use super::commercial::Commercial;
    pub use super::disconnect::Disconnect;
    pub use super::emote_only::EmoteOnly;
    pub use super::emote_only_off::EmoteOnlyOff;
    pub use super::followers::Followers;
    pub use super::followers_off::FollowersOff;
    pub use super::give_mod::GiveMod;
    pub use super::help::Help;
    pub use super::host::Host;
    pub use super::join::Join;
    pub use super::jtv_command::JtvCommand;
    pub use super::marker::Marker;
    pub use super::me::Me;
    pub use super::mods::Mods;
    pub use super::part::Part;
    pub use super::ping::Ping;
    pub use super::pong::Pong;
    pub use super::privmsg::Privmsg;
    pub use super::r9k_beta::R9kBeta;
    pub use super::r9k_beta_off::R9kBetaOff;
    pub use super::raid::Raid;
    pub use super::raw::Raw;
    pub use super::slow::Slow;
    pub use super::slow_off::SlowOff;
    pub use super::subscribers::Subscribers;
    pub use super::subscribers_off::SubscribersOff;
    pub use super::timeout::Timeout;
    pub use super::unban::Unban;
    pub use super::unhost::Unhost;
    pub use super::unmod::Unmod;
    pub use super::unraid::Unraid;
    pub use super::untimeout::Untimeout;
    pub use super::unvip::Unvip;
    pub use super::vip::Vip;
    pub use super::vips::Vips;
    pub use super::whisper::Whisper;
}

mod ban;
pub use ban::ban;

mod clear;
pub use clear::clear;

mod color;
pub use color::color;

mod command;
pub use command::command;

mod commercial;
pub use commercial::commercial;

mod disconnect;
pub use disconnect::disconnect;

mod emote_only;
pub use emote_only::emote_only;

mod emote_only_off;
pub use emote_only_off::emote_only_off;

mod followers;
pub use followers::followers;

mod followers_off;
pub use followers_off::followers_off;

mod give_mod;
pub use give_mod::give_mod;

mod help;
pub use help::help;

mod host;
pub use host::host;

mod join;
pub use join::join;

mod jtv_command;
pub use jtv_command::jtv_command;

mod marker;
pub use marker::marker;

mod me;
pub use me::me;

mod mods;
pub use mods::mods;

mod part;
pub use part::part;

mod ping;
pub use ping::ping;

mod pong;
pub use pong::pong;

mod privmsg;
pub use privmsg::privmsg;

mod r9k_beta;
pub use r9k_beta::r9k_beta;

mod r9k_beta_off;
pub use r9k_beta_off::r9k_beta_off;

mod raid;
pub use raid::raid;

mod raw;
pub use raw::raw;

mod slow;
pub use slow::slow;

mod slow_off;
pub use slow_off::slow_off;

mod subscribers;
pub use subscribers::subscribers;

mod subscribers_off;
pub use subscribers_off::subscribers_off;

mod timeout;
pub use timeout::timeout;

mod unban;
pub use unban::unban;

mod unhost;
pub use unhost::unhost;

mod unmod;
pub use unmod::unmod;

mod unraid;
pub use unraid::unraid;

mod untimeout;
pub use untimeout::untimeout;

mod unvip;
pub use unvip::unvip;

mod vip;
pub use vip::vip;

mod vips;
pub use vips::vips;

mod whisper;
pub use whisper::whisper;

struct ByteWriter<'a, W: Write + ?Sized>(&'a mut W);
impl<'a, W: Write + ?Sized> ByteWriter<'a, W> {
    fn new(writer: &'a mut W) -> Self {
        Self(writer)
    }

    fn channel(&mut self, data: impl AsRef<[u8]>) -> Result<()> {
        let data = data.as_ref();
        if !data.starts_with(b"#") {
            self.0.write_all(b"#")?;
        }
        self.0.write_all(data)
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

    fn command(mut self, channel: impl AsRef<[u8]>, parts: &[&dyn AsRef<[u8]>]) -> Result<()> {
        self.0.write_all(b"PRIVMSG ")?;
        self.channel(channel)?;
        self.0.write_all(b" :")?;
        self.parts(parts)
    }
}

macro_rules! serde_stuff {
    (@one $($x:tt)*) => { () };
    (@len $($e:expr),*) => { <[()]>::len(&[$(serde_stuff!(@one $e)),*]); };

    ($($ty:ident { $($field:ident),* $(,)?});* $(;)?) => {
        $(
            #[cfg(feature = "serde")]
            impl<'a> ::serde::Serialize for $crate::ng::commands::types::$ty<'a> {
                fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
                where
                    S: ::serde::Serializer,
                {
                    use $crate::ng::Encodable as _;
                    use ::serde::ser::{SerializeStruct as _, Error};

                    let mut data = vec![];
                    self.encode(&mut data).map_err(Error::custom)?;
                    let raw = std::str::from_utf8(&data).map_err(Error::custom)?;

                    let len = serde_stuff!(@len $($field),*);

                    let mut s = serializer.serialize_struct(stringify!($ty), std::cmp::max(len, 1))?;
                    s.serialize_field("raw", raw)?;
                    $(
                        s.serialize_field(stringify!($field), &self.$field)?;
                    )*
                    s.end()
                }
            }
        )*
    };
}

serde_stuff! {
    Ban { channel, username, reason };
    Clear { channel };
    Color { color };
    Command { channel, data };
    JtvCommand { data };
    Commercial { channel, length };
    Disconnect { };
    EmoteOnly { channel };
    EmoteOnlyOff { channel };
    Followers { channel, duration };
    FollowersOff { channel };
    GiveMod { channel, username };
    Help { channel };
    Host { source, target };
    Join { channel };
    Marker { channel, comment };
    Me { channel, msg };
    Mods { channel };
    Ping { token };
    Part { channel };
    Pong { token };
    Privmsg { channel, data };
    R9kBeta { channel };
    R9kBetaOff { channel };
    Raid { source, target };
    Raw { data };
    Slow { channel, duration };
    SlowOff { channel };
    Subscribers { channel };
    SubscribersOff { channel };
    Timeout { channel, username, duration, reason };
    Unban { channel, username };
    Unhost { channel };
    Unmod { channel, username };
    Unraid { channel };
    Untimeout { channel, username };
    Unvip { channel, username };
    Vip { channel, username };
    Vips { channel };
    Whisper { username, message };
}

#[cfg(test)]
fn test_encode(
    enc: impl super::Encodable,
    expected: impl for<'a> PartialEq<&'a str> + std::fmt::Debug,
) {
    let mut data = vec![];
    enc.encode(&mut data).unwrap();
    assert_eq!(expected, std::str::from_utf8(&data).unwrap());
}

#[cfg(all(test, feature = "serde"))]
fn test_serde<'de: 't, 't, T>(enc: T, expected: impl for<'a> PartialEq<&'a str> + std::fmt::Debug)
where
    T: ::serde::Serialize + super::Encodable,
    T: PartialEq + std::fmt::Debug,
    T: ::serde::Deserialize<'de> + 't,
{
    let json = serde_json::to_string_pretty(&enc).unwrap();

    #[derive(Debug, PartialEq, ::serde::Deserialize)]
    struct Wrapper {
        raw: String,
    }

    let wrapper: Wrapper = serde_json::from_str(&json).unwrap();
    assert_eq!(expected, &*wrapper.raw);

    // said json doesn't live for long enough
    // okay.
    let whatever: &'static str = Box::leak(json.into_boxed_str());

    let out = serde_json::from_str::<T>(&whatever).unwrap();
    assert_eq!(out, enc);
}
