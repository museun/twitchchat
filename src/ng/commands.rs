use std::io::{Result, Write};

macro_rules! export_commands {
    ($($ident:ident => $ty:ident)*) => {
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

macro_rules! serde_for_commands {
    (@one $($x:tt)*) => { () };
    (@len $($e:expr),*) => { <[()]>::len(&[$(serde_for_commands!(@one $e)),*]); };

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
fn test_serde<'de, T>(enc: T, expected: impl for<'a> PartialEq<&'a str> + std::fmt::Debug)
where
    T: super::Encodable + PartialEq + std::fmt::Debug,
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
