use twitchchat::messages::*;
use twitchchat::{AsOwned as _, Parse as _};

fn main() {
    let input = "@badge-info=subscriber/8;color=#59517B;tmi-sent-ts=1580932171144;user-type= :tmi.twitch.tv USERNOTICE #justinfan1234\r\n";

    // parse potentionally many messages from the input (flatten just safely unwraps the result)
    // msg is a decode::Message<'a> here
    for msg in twitchchat::decode(&input).flatten() {
        // parse message into a specific type
        let user_notice = UserNotice::parse(&msg).unwrap();
        // create an owned ('static) version of the message
        let owned: UserNotice<'static> = user_notice.as_owned();
        assert_eq!(user_notice, owned);

        // or parse the message into a 'All' type
        match AllCommands::parse(&msg).unwrap() {
            AllCommands::UserNotice(notice) => {
                // user_notice is a messages::UserNotice here
                assert_eq!(user_notice, notice);
            }
            _ => {}
        }

        // the tags are parsed and are accessible as methods
        // colors can be parsed into rgb/named types
        assert_eq!(
            user_notice.color().unwrap(),
            "#59517B".parse::<twitchchat::color::Color>().unwrap()
        );

        // you can manually get tags from the message
        let ts = user_notice.tags.get("tmi-sent-ts").unwrap();
        assert_eq!(ts, "1580932171144");

        // or as a type
        let ts = user_notice
            .tags
            .get_parsed::<_, u64>("tmi-sent-ts")
            .unwrap();
        assert_eq!(ts, 1580932171144);
    }

    // parse one message at a time
    // this returns the index of the start of the possible next message
    let input =
        ":tmi.twitch.tv PING 1234567\r\n:museun!museun@museun.tmi.twitch.tv JOIN #museun\r\n";

    let (d, left) = twitchchat::decode_one(input).unwrap();
    assert!(d > 0);
    assert_eq!(left.command, "PING");

    // use the new index
    let (i, right) = twitchchat::decode_one(&input[d..]).unwrap();
    assert_eq!(i, 0);
    assert_eq!(right.command, "JOIN");
}
