use super::*;

/// Message sent by a user
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Whisper<'t> {
    /// User who sent this messages
    pub name: Cow<'t, str>,
    /// Data that the user provided
    pub data: Cow<'t, str>,
    /// Tags attached to the message
    pub tags: Tags<'t>,
}

impl<'t> Whisper<'t> {
    /// Badges attached to this message
    ///    
    pub fn badges(&'t self) -> Vec<crate::Badge<'t>> {
        self.tags
            .get("badges")
            .map(|s| crate::parse_badges(s))
            .unwrap_or_default()
    }

    /// The color of the user who sent this message, if set
    pub fn color(&self) -> Option<crate::color::Color> {
        self.tags.get_parsed("color")
    }

    /// display_name
    pub fn display_name(&'t self) -> Option<&'t Cow<'t, str>> {
        self.tags.get("display-name")
    }

    /// Emotes attached to this message
    pub fn emotes(&self) -> Vec<crate::Emotes> {
        self.tags
            .get("emotes")
            .map(|s| crate::parse_emotes(s))
            .unwrap_or_default()
    }

    /// Whether the user sending this message was a staff member
    pub fn is_staff(&self) -> bool {
        self.badges()
            .iter()
            .any(|x| x.kind == crate::BadgeKind::Staff)
    }

    /// Whether the user sending this message had turbo
    pub fn is_turbo(&self) -> bool {
        self.badges()
            .iter()
            .any(|x| x.kind == crate::BadgeKind::Turbo)
    }

    /// Whether the user sending this message was a global moderator
    pub fn is_global_moderator(&self) -> bool {
        self.badges()
            .iter()
            .any(|x| x.kind == crate::BadgeKind::GlobalMod)
    }

    /// The timestamp of when this message was received by Twitch
    pub fn tmi_sent_ts(&self) -> Option<u64> {
        self.tags.get_parsed("tmi-sent-ts")
    }

    /// The id of the user who sent this message
    pub fn user_id(&self) -> Option<u64> {
        self.tags.get_parsed("user-id")
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Whisper<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("WHISPER")?;
        Ok(Self {
            name: msg.expect_nick()?,
            data: msg.expect_data()?.clone(),
            tags: msg.tags.clone(),
        })
    }
}

impl<'t> AsOwned for Whisper<'t> {
    type Owned = Whisper<'static>;
    fn as_owned(&self) -> Self::Owned {
        Whisper {
            name: self.name.as_owned(),
            data: self.data.as_owned(),
            tags: self.tags.as_owned(),
        }
    }
}
