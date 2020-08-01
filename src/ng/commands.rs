#![cfg_attr(debug_assertions, allow(dead_code, unused_variables))]
use std::io::{Result, Write};

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

// #[cfg(feature = "serde")]
// struct SerdeWrapper<'a, T: 'a>(T, &'static str, std::marker::PhantomData<&'a T>);

// #[cfg(feature = "serde")]
// impl<'a, T: 'a> ::serde::Serialize for SerdeWrapper<'a, T>
// where
//     T: crate::ng::Encodable,
// {
//     fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
//     where
//         S: ::serde::Serializer,
//     {
//         use ::serde::ser::{Error, SerializeStruct as _};

//         let Self(item, name, ..) = self;

//         use crate::Enco
//         let mut data = vec![];
//         item.encode(&mut data).map_err(Error::custom)?;
//         let raw = std::str::from_utf8(&data).map_err(Error::custom)?;

//         let mut s = serializer.serialize_struct(name, 1)?;
//         s.serialize_field("raw", raw)?;
//         s.end()
//     }
// }

macro_rules! serde_stuff {
    (@one $($x:tt)*) => { () };
    (@len $($e:expr),*) => { <[()]>::len(&[$(serde_stuff!(@one $e)),*]); };

    ($($ty:ident { $($field:ident),* $(,)?});* $(;)?) => {
        $(
            #[cfg(feature = "serde")]
            impl<'a> ::serde::Serialize for $ty<'a> {
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

mod ban;
pub use ban::{ban, Ban};

mod clear;
pub use clear::{clear, Clear};

mod color;
pub use color::{color, Color};

mod command;
pub use command::{command, Command};

mod commercial;
pub use commercial::{commercial, Commercial};

mod disconnect;
pub use disconnect::{disconnect, Disconnect};

mod emote_only;
pub use emote_only::{emote_only, EmoteOnly};

mod emote_only_off;
pub use emote_only_off::{emote_only_off, EmoteOnlyOff};

mod followers;
pub use followers::{followers, Followers};

mod followers_off;
pub use followers_off::{followers_off, FollowersOff};

mod give_mod;
pub use give_mod::{give_mod, GiveMod};

mod help;
pub use help::{help, Help};

mod host;
pub use host::{host, Host};

mod join;
pub use join::{join, Join};

mod jtv_command;
pub use jtv_command::{jtv_command, JtvCommand};

mod marker;
pub use marker::{marker, Marker};

mod me;
pub use me::{me, Me};

mod mods;
pub use mods::{mods, Mods};

mod part;
pub use part::{part, Part};

mod ping;
pub use ping::{ping, Ping};

mod pong;
pub use pong::{pong, Pong};

mod privmsg;
pub use privmsg::{privmsg, Privmsg};

mod r9k_beta;
pub use r9k_beta::{r9k_beta, R9kBeta};

mod r9k_beta_off;
pub use r9k_beta_off::{r9k_beta_off, R9kBetaOff};

mod raid;
pub use raid::{raid, Raid};

mod raw;
pub use raw::{raw, Raw};

mod slow;
pub use slow::{slow, Slow};

mod slow_off;
pub use slow_off::{slow_off, SlowOff};

mod subscribers;
pub use subscribers::{subscribers, Subscribers};

mod subscribers_off;
pub use subscribers_off::{subscribers_off, SubscribersOff};

mod timeout;
pub use timeout::{timeout, Timeout};

mod unban;
pub use unban::{unban, Unban};

mod unhost;
pub use unhost::{unhost, Unhost};

mod unmod;
pub use unmod::{unmod, Unmod};

mod unraid;
pub use unraid::{unraid, Unraid};

mod untimeout;
pub use untimeout::{untimeout, Untimeout};

mod unvip;
pub use unvip::{unvip, Unvip};

mod vip;
pub use vip::{vip, Vip};

mod vips;
pub use vips::{vips, Vips};

mod whisper;
pub use whisper::{whisper, Whisper};
