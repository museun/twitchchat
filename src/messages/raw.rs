use super::*;

/// Raw IRC message
#[derive(Debug, Clone)]
pub struct Raw<T = String>
where
    T: StringMarker,
{
    /// The raw string
    pub raw: T,
    /// Any targets found in the message
    pub tags: Tags<T>,
    /// The prefix of the message
    pub prefix: Option<Prefix<T>>,
    /// The command of the message
    pub command: T,
    /// Arguments to the command
    pub args: T,
    /// Any data provided
    pub data: Option<T>,
}

impl<'a> TryFrom<&'a Message<&'a str>> for Raw<&'a str> {
    type Error = InvalidMessage;

    fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
        Ok(Self {
            raw: msg.raw,
            tags: msg.tags.clone(),
            prefix: msg.prefix,
            command: msg.command,
            args: msg.args,
            data: msg.data,
        })
    }
}

as_owned!(for Raw {
    raw,
    tags,
    prefix,
    command,
    args,
    data,
});
