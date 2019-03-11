/// IRC Prefix, not really used by Twitch once capabilities are enabled
#[derive(Debug, Clone, PartialEq)]
pub enum Prefix {
    /// User prefix. i.e nick!user@host
    User {
        nick: String,
        user: String,
        host: String,
    },
    /// Server prefix. i.e. tmi.twitch.tv
    Server { host: String },
}

impl Prefix {
    pub(super) fn parse(input: &str) -> Option<Self> {
        if !input.starts_with(':') {
            return None;
        }

        let s = input[1..input.find(' ').unwrap_or_else(|| input.len())].trim();
        match s.find('!') {
            Some(pos) => {
                let at = s.find('@')?;
                Some(Prefix::User {
                    nick: s[..pos].to_string(),
                    user: s[pos + 1..at].to_string(),
                    host: s[at + 1..].to_string(),
                })
            }
            None => Some(Prefix::Server {
                host: s.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_user() {
        let input = ":museun_32[asdf]!~this_is_a_user@0xDEADBEEF.irc.local.1234.host";
        assert_eq!(
            Prefix::parse(&input).unwrap(),
            Prefix::User {
                nick: "museun_32[asdf]".into(),
                user: "~this_is_a_user".into(),
                host: "0xDEADBEEF.irc.local.1234.host".into()
            }
        )
    }

    #[test]
    fn parse_missing() {
        let input = "no_leading_colon";
        assert_eq!(Prefix::parse(&input), None)
    }

    #[test]
    fn parse_server() {
        let input = ":jtv";
        assert_eq!(
            Prefix::parse(&input).unwrap(),
            Prefix::Server { host: "jtv".into() }
        );
        let input = ":tmi.twitch.tv";
        assert_eq!(
            Prefix::parse(&input).unwrap(),
            Prefix::Server {
                host: "tmi.twitch.tv".into()
            }
        );
        let input = ":irc.some.server.local.domain";
        assert_eq!(
            Prefix::parse(&input).unwrap(),
            Prefix::Server {
                host: "irc.some.server.local.domain".into()
            }
        );
    }
}
