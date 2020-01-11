use super::*;
#[test]
fn parse_user_prefix() {
    let prefix = Prefix::parse(":museun!museun@museun.tmi.twitch.tv").unwrap();
    assert_eq!(
        prefix,
        Prefix::User {
            nick: "museun".into(),
        },
    )
}

#[test]
fn parse_server_prefix() {
    let prefix = Prefix::parse(":tmi.twitch.tv").unwrap();
    assert_eq!(
        prefix,
        Prefix::Server {
            host: "tmi.twitch.tv".into(),
        },
    );

    let prefix = Prefix::parse("tmi.twitch.tv").unwrap();
    assert_eq!(
        prefix,
        Prefix::Server {
            host: "tmi.twitch.tv".into(),
        },
    )
}

#[test]
fn missing_colon_prefix() {
    for input in &["museun!museun@museun.tmi.twitch.tv", "not_tmi.twitch.tv"] {
        assert!(Prefix::parse(input).is_none());
    }
}

#[test]
fn decode_one() {
    let input = ":foo!bar@baz PRIVMSG #test :this is a test\r\n:local.host PING :1234\r\n";
    let (next, _msg) = super::decode_one(input).unwrap();
    assert!(next > 0);

    // this should be the last message
    let (next, _msg) = super::decode_one(&input[next..]).unwrap();
    assert_eq!(next, 0);

    // try with a bad element at the end
    let input = ":foo!bar@baz PRIVMSG #test :this is a test\r\n:local.host PING :1234\r\nfoo";
    {
        let (next, _msg) = super::decode_one(input).unwrap();
        assert!(next > 0);

        let input = &input[next..];
        let (next, _msg) = super::decode_one(&input).unwrap();
        assert!(next > 0);

        // last one should be an error
        let input = &input[next..];
        super::decode_one(&input).unwrap_err();
    }
}

#[test]
fn decode() {
    let input = ":foo!bar@baz PRIVMSG #test :this is a test\r\n:local.host PING :1234\r\nfoo";

    // try with the iterator
    let mut vec = super::decode(input).collect::<Vec<_>>();
    assert_eq!(vec.len(), 3);

    // last one should be an error
    vec.pop().unwrap().unwrap_err();
    // rest should be okay
    while let Some(ok) = vec.pop() {
        ok.unwrap();
    }

    // remove all of the bad ones, only keep the 'success'
    let vec = super::decode(input).flatten().collect::<Vec<_>>();
    assert_eq!(vec.len(), 2);
}

#[test]
fn cap_ack() {
    let input = "tmi.twitch.tv CAP * ACK :twitch.tv/commands\r\n";
    let msg = Message::parse(input).unwrap();
    assert_eq!(
        msg.prefix.unwrap(),
        Prefix::Server {
            host: "tmi.twitch.tv"
        }
    );
    assert_eq!(msg.command, "CAP");
    assert_eq!(msg.args, "* ACK");
    assert_eq!(msg.data.unwrap(), "twitch.tv/commands")
}
