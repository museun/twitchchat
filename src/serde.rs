use crate::{FromIrcMessage, IrcMessage, Str};

use serde::{
    de::{Error, MapAccess, Visitor},
    Deserialize, Deserializer,
};

use std::marker::PhantomData;

impl<'de, 'a> Deserialize<'de> for Str<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <Box<str>>::deserialize(deserializer).map(Str::Owned)
    }
}

pub struct RawVisitor<'a, T>(PhantomData<&'a T>);

impl<'a, T> Default for RawVisitor<'a, T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<'de, 'a, T> Visitor<'de> for RawVisitor<'a, T>
where
    T: FromIrcMessage<'a>,
    T::Error: std::error::Error,
{
    type Value = T;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        const RAW: &str = "raw";

        let mut raw = None;
        while let Some(key) = map.next_key::<&str>()? {
            if key != RAW {
                map.next_value::<serde::de::IgnoredAny>()?;
                continue;
            }
            let val = map.next_value::<Str<'a>>()?;
            if raw.replace(val).is_some() {
                return Err(A::Error::duplicate_field(RAW));
            }
        }

        let raw = raw.ok_or_else(|| A::Error::missing_field(RAW))?;
        let irc = IrcMessage::parse(raw).map_err(|err| {
            A::Error::custom(format!(
                "cannot parse '{}' from the irc message: {}",
                std::any::type_name::<T>(),
                err
            ))
        })?;

        T::from_irc(irc).map_err(|err| {
            A::Error::custom(format!(
                "cannot parse '{}' because: {}",
                std::any::type_name::<T>(),
                err
            ))
        })
    }
}

#[cfg(test)]
pub(crate) fn round_trip_json<'a, T>(input: &'a str)
where
    T: FromIrcMessage<'a> + PartialEq + std::fmt::Debug,
    T::Error: std::fmt::Debug,
    for<'de> T: ::serde::Serialize + ::serde::Deserialize<'de>,
{
    let (_, msg) = crate::irc::parse_one(input).unwrap();
    let left = T::from_irc(msg).unwrap();
    let json = serde_json::to_string(&left).unwrap();
    let right = serde_json::from_str::<T>(&json).unwrap();
    assert_eq!(left, right)
}

#[cfg(test)]
pub(crate) fn round_trip_rmp<'a, T>(input: &'a str)
where
    T: FromIrcMessage<'a> + PartialEq + std::fmt::Debug,
    T::Error: std::fmt::Debug,
    for<'de> T: ::serde::Serialize + ::serde::Deserialize<'de>,
{
    let (_, msg) = crate::irc::parse_one(input).unwrap();
    let left = T::from_irc(msg).unwrap();
    let vec = rmp_serde::to_vec(&left).unwrap();
    let right = rmp_serde::from_slice::<T>(&vec).unwrap();
    assert_eq!(left, right)
}
