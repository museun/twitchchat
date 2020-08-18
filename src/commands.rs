//! Provides [`Encodable`][encodable] commands.
//!
//! These can be used with an encoder, or with the [`Encodable`][encodable] trait directly to write typed messages.
//!
//! The [functions][functions] in this module produce borrowed types in the [`types`][types] module. You can store the [`types`][types] for multiple encodings.
//!
//! ### Some provided encoders:
//! * [`AsyncEncoder`](../struct.AsyncEncoder.html)
//! * [`Encoder`](../struct.Encoder.html)
//! * [`MpscWriter`](../writer/struct.MpscWriter.html)
//! * [`AsyncWriter`](../writer/struct.AsyncWriter.html)
//!
//! [functions]: ./index.html#functions
//! [types]: ./types/index.html
//! [encodable]: ./trait.Encodable.html
pub use super::Encodable;

macro_rules! write_cmd {
    ($w:expr, $chan:expr => $data:expr) => {{
        write!($w, "PRIVMSG {} :", $chan)?;
        write!($w, "{}", $data)?;
        write!($w, "\r\n")
    }};

    ($w:expr, $chan:expr => $fmt:expr, $($args:expr),* $(,)?) => {{
        write!($w, "PRIVMSG {} :", $chan)?;
        write!($w, $fmt, $($args),*)?;
        write!($w, "\r\n")
    }};
}

macro_rules! write_jtv_cmd {
    ($w:expr, $fmt:expr) => {
        write_cmd!($w, "jtv" => $fmt)
    };

    ($w:expr, $fmt:expr, $($args:expr),* $(,)?) => {
        write_cmd!($w, "jtv" => $fmt, $($args),*)
    };
}

macro_rules! write_nl {
    ($w:expr, $fmt:expr, $($args:expr),* $(,)?) => {{
        write!($w, $fmt, $($args),*)?;
        write!($w, "\r\n")
    }};
}

macro_rules! export_commands {
    ($($ident:ident => $ty:ident)*) => {
        /// Concrete types produced by the functions in the `commands` module.
        ///
        /// e.g. `join("#museun") -> Join<'_>` (where its borrowed from the input `&str`)
        pub mod types {
            $( pub use super::$ident::$ty; )*
        }
        $(
            mod $ident;
            pub use $ident::$ident;
        )*
    };
}

export_commands! {
    ban             => Ban
    clear           => Clear
    color           => Color
    command         => Command
    commercial      => Commercial
    disconnect      => Disconnect
    emote_only      => EmoteOnly
    emote_only_off  => EmoteOnlyOff
    followers       => Followers
    followers_off   => FollowersOff
    give_mod        => GiveMod
    help            => Help
    host            => Host
    join            => Join
    jtv_command     => JtvCommand
    marker          => Marker
    me              => Me
    mods            => Mods
    part            => Part
    ping            => Ping
    pong            => Pong
    privmsg         => Privmsg
    r9k_beta        => R9kBeta
    r9k_beta_off    => R9kBetaOff
    raid            => Raid
    raw             => Raw
    register        => Register
    reply           => Reply
    slow            => Slow
    slow_off        => SlowOff
    subscribers     => Subscribers
    subscribers_off => SubscribersOff
    timeout         => Timeout
    unban           => Unban
    unhost          => Unhost
    unmod           => Unmod
    unraid          => Unraid
    untimeout       => Untimeout
    unvip           => Unvip
    vip             => Vip
    vips            => Vips
    whisper         => Whisper
}

macro_rules! serde_for_commands {
    (@one $($x:tt)*) => { () };
    (@len $($e:expr),*) => { <[()]>::len(&[$(serde_for_commands!(@one $e)),*]); };

    ($($ty:ident { $($field:ident),* $(,)?});* $(;)?) => {
        $(
            #[cfg(feature = "serde")]
            impl<'a> ::serde::Serialize for $crate::commands::types::$ty<'a> {
                fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
                where
                    S: ::serde::Serializer,
                {
                    use $crate::Encodable as _;
                    use ::serde::ser::{SerializeStruct as _, Error};

                    let mut data = vec![];
                    self.encode(&mut data).map_err(Error::custom)?;
                    let raw = std::str::from_utf8(&data).map_err(Error::custom)?;

                    let len = serde_for_commands!(@len $($field),*);

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

serde_for_commands! {
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
    Privmsg { channel, msg };
    R9kBeta { channel };
    R9kBetaOff { channel };
    Raid { source, target };
    Raw { data };
    Register { user_config };
    Reply { channel, msg_id, msg };
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

pub(crate) trait Length {
    fn length(&self) -> usize;
}

impl Length for str {
    fn length(&self) -> usize {
        self.len()
    }
}

impl Length for &str {
    fn length(&self) -> usize {
        self.len()
    }
}

use std::{fmt::Display, ops::Deref};
impl<T> Length for Option<T>
where
    T: Length + Deref<Target = str>,
{
    fn length(&self) -> usize {
        self.as_deref().map(Length::length).unwrap_or(0)
    }
}

pub(crate) struct MaybeEmpty<T>(pub Option<T>);

impl<T> Display for MaybeEmpty<T>
where
    T: Display + Length,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(d) if d.length() > 0 => write!(f, " {}", d),
            _ => Ok(()),
        }
    }
}

pub(crate) struct Channel<'a>(pub &'a str);

impl<'a> Display for Channel<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.starts_with('#') {
            write!(f, "{}", self.0)
        } else {
            write!(f, "#{}", self.0)
        }
    }
}

#[cfg(test)]
fn test_encode(enc: impl Encodable, expected: impl for<'a> PartialEq<&'a str> + std::fmt::Debug) {
    let mut data = vec![];
    enc.encode(&mut data).unwrap();
    assert_eq!(expected, std::str::from_utf8(&data).unwrap());
}

#[cfg(all(test, feature = "serde"))]
fn test_serde<'de, T>(enc: T, expected: impl for<'a> PartialEq<&'a str> + std::fmt::Debug)
where
    T: Encodable + PartialEq + std::fmt::Debug,
    T: ::serde::Serialize + ::serde::Deserialize<'de>,
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
