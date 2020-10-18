use twitchchat::{
    messages,
    //  for `from_irc()`
    FromIrcMessage as _,
    // for into_owned()
    IntoOwned as _,
};

fn main() {
    // show off the low-level parser
    parse_demo();

    // this provides a 'reader'/'iterator' instead of just a boring parser
    decoder_demo();

    // and block on a future for the async decoder
    futures_lite::future::block_on(decoder_async_demo())
}

fn parse_demo() {
    let input =
        "@key1=val1;key2=true;key3=42 :sender!sender@server PRIVMSG #museun :this is a test\r\n";

    // you can get an iterator of messages that borrow from the input string
    for msg in twitchchat::irc::parse(input) {
        let msg: twitchchat::IrcMessage<'_> = msg.unwrap();
        // you can get the raw string back
        assert_eq!(msg.get_raw(), input);

        // you can parse it into a specific type. e.g. a PRIVMSG.
        // this continues to borrow from the original string slice
        let pm = messages::Privmsg::from_irc(msg).unwrap();
        assert_eq!(pm.channel(), "#museun");

        // you can consume the parsed message to get the raw string back out.
        // this gives you a MaybeOwned<'a> because the type can be converted to an owned state (e.g. static);
        let msg: twitchchat::maybe_owned::MaybeOwned<'_> = pm.into_inner();

        // `MaybeOwned<'a>` can be used as a `&'a str`.
        let msg = twitchchat::irc::parse(&*msg)
            .next()
            .map(|s| s.unwrap())
            .unwrap();

        // parse it as an Commands, which wraps all of the provided messages
        let all = messages::Commands::from_irc(msg).unwrap();
        assert!(matches!(all, messages::Commands::Privmsg{..}));

        // this is still borrowing from the 'input' from above.
        let all: messages::Commands<'_> = all;

        // to turn it into an 'owned' version (e.g. a 'static lifetime)
        let all = all.into_owned();
        let _all: messages::Commands<'static> = all;
    }

    // double the string for the test
    let old_len = input.len();
    let input = input.repeat(2);

    // you can also parse a 'single' message in a streaming fashion
    // this returns a pos > 0 if the index of the start of the next possible message
    let (pos, msg_a) = twitchchat::irc::parse_one(&input).unwrap();
    assert_eq!(pos, old_len);

    // and parse the rest of the message
    // this returns a pos if 0 if this was the last message
    let (pos, msg_b) = twitchchat::irc::parse_one(&input[pos..]).unwrap();
    assert_eq!(pos, 0);

    // and it should've parsed the same message twice
    assert_eq!(msg_a, msg_b);

    // and you can get the a tags 'view' from the message, if any tags were provided
    let msg = messages::Privmsg::from_irc(msg_a).unwrap();
    // you can get the string value for a key
    assert_eq!(msg.tags().get("key1").unwrap(), "val1");
    // or it as a 'truthy' value
    assert_eq!(msg.tags().get_as_bool("key2"), true);
    // or as a FromStr parsed value
    assert_eq!(msg.tags().get_parsed::<_, i32>("key3").unwrap(), 42);

    // you can convert a parsed message into an Commands easily by using From/Into;
    let all: messages::Commands<'_> = msg_b.into();
    assert!(matches!(all, messages::Commands::Raw{..}));
}

fn decoder_demo() {
    let input =
        "@key1=val1;key2=true;key3=42 :sender!sender@server PRIVMSG #museun :this is a test\r\n";

    let source = input.repeat(5);
    // Cursor<Vec<u8>> impl std::io::Read. using it for this demo
    let reader = std::io::Cursor::new(source.into_bytes());

    // you can make a decoder over an std::io::Read
    let mut decoder = twitchchat::Decoder::new(reader);

    // you use use read_message than the 'msg' is borrowed until the next call of 'read_message'
    while let Ok(_msg) = decoder.read_message() {
        // msg is borrowed from the decoder here
    }

    // you can get the inner reader out
    let mut reader = decoder.into_inner();
    // seek back to the beginning for this demo
    reader.set_position(0);

    {
        // you can also just give it a &mut Reader
        let _decoder = twitchchat::Decoder::new(&mut reader);
        // which will drop the decoder here and you'll still have the 'reader' from above
    }

    // the decoder is also an iterator.
    // when using the iterator you'll get an 'owned' message back.
    for msg in twitchchat::Decoder::new(&mut reader) {
        // and msg is owned here ('static)
        // error if it failed to parse, or an IO error.
        let _msg: messages::IrcMessage<'static> = msg.unwrap();
    }
}

// all of the Sync is also applicable to the Async version.
async fn decoder_async_demo() {
    use futures_lite::StreamExt as _; // for 'next' on the Stream

    let input =
        "@key1=val1;key2=true;key3=42 :sender!sender@server PRIVMSG #museun :this is a test\r\n";

    let source = input.repeat(5);
    // Cursor<Vec<u8>> impl std::io::Read. using it for this demo
    let reader = futures_lite::io::Cursor::new(source.into_bytes());

    // you can make a decoder over an std::io::Read
    let mut decoder = twitchchat::AsyncDecoder::new(reader);

    // you use use read_message than the 'msg' is borrowed until the next call of 'read_message'
    while let Ok(_msg) = decoder.read_message().await {
        // msg is borrowed from the decoder here
    }

    // you can get the inner reader out
    let mut reader = decoder.into_inner();
    // seek back to the beginning for this demo
    reader.set_position(0);

    {
        // you can also just give it a &mut Reader
        let _decoder = twitchchat::AsyncDecoder::new(&mut reader);
        // which will drop the decoder here and you'll still have the 'reader' from above
    }

    // the decoder is also an Stream.
    // when using the Stream you'll get an 'owned' message back.
    while let Some(msg) = twitchchat::AsyncDecoder::new(&mut reader).next().await {
        // and msg is owned here ('static)
        // error if it failed to parse, or an IO error.
        let _msg: messages::IrcMessage<'static> = msg.unwrap();
    }
}
