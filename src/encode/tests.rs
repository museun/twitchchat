use super::*;

fn test_encode(
    mut func: impl FnMut(&mut Encoder<Vec<u8>>) -> Result<(), crate::Error>,
    expected: impl AsRef<str>,
) {
    let mut encoder = Encoder::new(vec![]);
    func(&mut encoder).unwrap();
    assert_eq!(
        std::str::from_utf8(&encoder.into_inner()).unwrap(),
        expected.as_ref()
    );
}

#[test]
fn encode_raw() {
    test_encode(
        |enc| enc.raw("PRIVMSG #test :this is a test"),
        "PRIVMSG #test :this is a test\r\n",
    );
}

#[test]
fn encode_pong() {
    test_encode(|enc| enc.pong("123456789"), "PONG :123456789\r\n");
}

#[test]
fn encode_ping() {
    test_encode(|enc| enc.ping("123456789"), "PING 123456789\r\n");
}

#[test]
fn encode_join() {
    test_encode(|enc| enc.join("#museun"), "JOIN #museun\r\n");
}

#[test]
fn encode_part() {
    test_encode(|enc| enc.part("#museun"), "PART #museun\r\n");
}

#[test]
fn encode_privmsg() {
    test_encode(
        |enc| enc.privmsg("#museun", "this is a test of a line"),
        "PRIVMSG #museun :this is a test of a line\r\n",
    );

    test_encode(
        |enc| enc.privmsg("#museun", &"foo ".repeat(500)),
        format!("PRIVMSG #museun :{}\r\n", &"foo ".repeat(500)),
    );
}

#[test]
fn encode_ban() {
    test_encode(
        |enc| enc.ban("#museun", "museun", None),
        "PRIVMSG #museun :/ban museun\r\n",
    )
}

#[test]
fn encode_clear() {
    test_encode(|enc| enc.clear("#museun"), "PRIVMSG #museun :/clear\r\n")
}

#[test]
fn encode_color() {
    let blue = "blue".parse().unwrap();
    test_encode(
        |enc| enc.color(blue),
        format!("PRIVMSG jtv :/color {}\r\n", blue),
    )
}

#[test]
fn encode_command() {
    test_encode(
        |enc| enc.command("#museun", "/testing"),
        "PRIVMSG #museun :/testing\r\n",
    )
}

#[test]
fn encode_commercial() {
    test_encode(
        |enc| enc.commercial("#museun", None),
        "PRIVMSG #museun :/commercial\r\n",
    );
    test_encode(
        |enc| enc.commercial("#museun", 10),
        "PRIVMSG #museun :/commercial 10\r\n",
    );
    test_encode(
        |enc| enc.commercial("#museun", Some(10)),
        "PRIVMSG #museun :/commercial 10\r\n",
    );
}

#[test]
fn encode_disconnect() {
    test_encode(|enc| enc.disconnect(), "PRIVMSG jtv :/disconnect\r\n")
}

#[test]
fn encode_emoteonly() {
    test_encode(
        |enc| enc.emote_only("#museun"),
        "PRIVMSG #museun :/emoteonly\r\n",
    )
}

#[test]
fn encode_emoteonlyoff() {
    test_encode(
        |enc| enc.emote_only_off("#museun"),
        "PRIVMSG #museun :/emoteonlyoff\r\n",
    )
}

#[test]
fn encode_followers() {
    test_encode(
        |enc| enc.followers("#museun", "1 week"),
        "PRIVMSG #museun :/followers 1 week\r\n",
    )
}

#[test]
fn encode_followersoff() {
    test_encode(
        |enc| enc.followers_off("#museun"),
        "PRIVMSG #museun :/followersoff\r\n",
    )
}

#[test]
fn encode_help() {
    test_encode(|enc| enc.help("#museun"), "PRIVMSG #museun :/help\r\n")
}

#[test]
fn encode_host() {
    test_encode(
        |enc| enc.host("#museun", "#shaken_bot"),
        "PRIVMSG #museun :/host #shaken_bot\r\n",
    )
}

#[test]
fn encode_marker() {
    test_encode(
        |enc| enc.marker("#museun", Some("this is an example")),
        "PRIVMSG #museun :/marker this is an example\r\n",
    );
    test_encode(
        |enc| enc.marker("#museun", "this is an example"),
        "PRIVMSG #museun :/marker this is an example\r\n",
    );
    test_encode(
        |enc| enc.marker("#museun", "a".repeat(200).as_str()),
        format!("PRIVMSG #museun :/marker {}\r\n", "a".repeat(140)),
    );
    test_encode(
        |enc| enc.marker("#museun", None),
        "PRIVMSG #museun :/marker\r\n",
    );
}

#[test]
fn encode_me() {
    test_encode(
        |enc| enc.me("#museun", "some emote"),
        "PRIVMSG #museun :/me some emote\r\n",
    );
}

#[test]
fn encode_give_mod() {
    test_encode(
        |enc| enc.give_mod("#museun", "shaken_bot"),
        "PRIVMSG #museun :/mod shaken_bot\r\n",
    )
}

#[test]
fn encode_mods() {
    test_encode(|enc| enc.mods("#museun"), "PRIVMSG #museun :/mods\r\n")
}

#[test]
fn encode_r9kbeta() {
    test_encode(
        |enc| enc.r9k_beta("#museun"),
        "PRIVMSG #museun :/r9kbeta\r\n",
    )
}

#[test]
fn encode_r9kbetaoff() {
    test_encode(
        |enc| enc.r9k_beta_off("#museun"),
        "PRIVMSG #museun :/r9kbetaoff\r\n",
    )
}

#[test]
fn encode_raid() {
    test_encode(
        |enc| enc.raid("#museun", "#museun"),
        "PRIVMSG #museun :/raid #museun\r\n",
    )
}

#[test]
fn encode_slow() {
    test_encode(
        |enc| enc.slow("#museun", Some(42)),
        "PRIVMSG #museun :/slow 42\r\n",
    );
    test_encode(
        |enc| enc.slow("#museun", 42),
        "PRIVMSG #museun :/slow 42\r\n",
    );
    test_encode(
        |enc| enc.slow("#museun", None),
        "PRIVMSG #museun :/slow 120\r\n",
    );
}

#[test]
fn encode_slowoff() {
    test_encode(
        |enc| enc.slow_off("#museun"),
        "PRIVMSG #museun :/slowoff\r\n",
    )
}

#[test]
fn encode_subscribers() {
    test_encode(
        |enc| enc.subscribers("#museun"),
        "PRIVMSG #museun :/subscribers\r\n",
    )
}

#[test]
fn encode_subscribersoff() {
    test_encode(
        |enc| enc.subscribers_off("#museun"),
        "PRIVMSG #museun :/subscribersoff\r\n",
    )
}

#[test]
fn encode_timeout() {
    test_encode(
        |enc| enc.timeout("#museun", "museun", None, None),
        "PRIVMSG #museun :/timeout museun\r\n",
    );
    test_encode(
        |enc| enc.timeout("#museun", "museun", Some("1d2h"), None),
        "PRIVMSG #museun :/timeout museun 1d2h\r\n",
    );
    test_encode(
        |enc| enc.timeout("#museun", "museun", None, Some("spamming")),
        "PRIVMSG #museun :/timeout museun spamming\r\n",
    );
    test_encode(
        |enc| enc.timeout("#museun", "museun", Some("1d2h"), Some("spamming")),
        "PRIVMSG #museun :/timeout museun 1d2h spamming\r\n",
    );
}

#[test]
fn encode_unban() {
    test_encode(
        |enc| enc.unban("#museun", "museun"),
        "PRIVMSG #museun :/unban museun\r\n",
    )
}

#[test]
fn encode_unhost() {
    test_encode(|enc| enc.unhost("#museun"), "PRIVMSG #museun :/unhost\r\n")
}

#[test]
fn encode_unmod() {
    test_encode(
        |enc| enc.unmod("#museun", "museun"),
        "PRIVMSG #museun :/unmod museun\r\n",
    )
}

#[test]
fn encode_unraid() {
    test_encode(|enc| enc.unraid("#museun"), "PRIVMSG #museun :/unraid\r\n")
}

#[test]
fn encode_untimeout() {
    test_encode(
        |enc| enc.untimeout("#museun", "museun"),
        "PRIVMSG #museun :/untimeout museun\r\n",
    )
}

#[test]
fn encode_unvip() {
    test_encode(
        |enc| enc.unvip("#museun", "museun"),
        "PRIVMSG #museun :/unvip museun\r\n",
    )
}

#[test]
fn encode_vip() {
    test_encode(
        |enc| enc.vip("#museun", "museun"),
        "PRIVMSG #museun :/vip museun\r\n",
    )
}

#[test]
fn encode_vips() {
    test_encode(|enc| enc.vips("#museun"), "PRIVMSG #museun :/vips\r\n")
}

#[test]
fn encode_whisper() {
    test_encode(
        |enc| enc.whisper("museun", "hello world"),
        "PRIVMSG jtv :/w museun hello world\r\n",
    )
}
