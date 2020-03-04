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
        |enc| enc.ban("museun", None),
        "PRIVMSG jtv :/ban museun\r\n",
    )
}

#[test]
fn encode_clear() {
    test_encode(|enc| enc.clear(), "PRIVMSG jtv :/clear\r\n")
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
    test_encode(|enc| enc.command("/testing"), "PRIVMSG jtv :/testing\r\n")
}

#[test]
fn encode_commercial() {
    test_encode(|enc| enc.commercial(None), "PRIVMSG jtv :/commercial\r\n");
    test_encode(|enc| enc.commercial(10), "PRIVMSG jtv :/commercial 10\r\n");
    test_encode(
        |enc| enc.commercial(Some(10)),
        "PRIVMSG jtv :/commercial 10\r\n",
    );
}

#[test]
fn encode_disconnect() {
    test_encode(|enc| enc.disconnect(), "PRIVMSG jtv :/disconnect\r\n")
}

#[test]
fn encode_emoteonly() {
    test_encode(|enc| enc.emote_only(), "PRIVMSG jtv :/emoteonly\r\n")
}

#[test]
fn encode_emoteonlyoff() {
    test_encode(|enc| enc.emote_only_off(), "PRIVMSG jtv :/emoteonlyoff\r\n")
}

#[test]
fn encode_followers() {
    test_encode(
        |enc| enc.followers("1 week"),
        "PRIVMSG jtv :/followers 1 week\r\n",
    )
}

#[test]
fn encode_followersoff() {
    test_encode(|enc| enc.followers_off(), "PRIVMSG jtv :/followersoff\r\n")
}

#[test]
fn encode_help() {
    test_encode(|enc| enc.help(), "PRIVMSG jtv :/help\r\n")
}

#[test]
fn encode_host() {
    test_encode(|enc| enc.host("#museun"), "PRIVMSG jtv :/host #museun\r\n")
}

#[test]
fn encode_marker() {
    test_encode(
        |enc| enc.marker(Some("this is an example")),
        "PRIVMSG jtv :/marker this is an example\r\n",
    );
    test_encode(
        |enc| enc.marker("this is an example"),
        "PRIVMSG jtv :/marker this is an example\r\n",
    );
    test_encode(|enc| enc.marker(None), "PRIVMSG jtv :/marker\r\n");
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
        |enc| enc.give_mod("#museun"),
        "PRIVMSG jtv :/mod #museun\r\n",
    )
}

#[test]
fn encode_mods() {
    test_encode(|enc| enc.mods(), "PRIVMSG jtv :/mods\r\n")
}

#[test]
fn encode_r9kbeta() {
    test_encode(|enc| enc.r9k_beta(), "PRIVMSG jtv :/r9kbeta\r\n")
}

#[test]
fn encode_r9kbetaoff() {
    test_encode(|enc| enc.r9k_beta_off(), "PRIVMSG jtv :/r9kbetaoff\r\n")
}

#[test]
fn encode_raid() {
    test_encode(|enc| enc.raid("#museun"), "PRIVMSG jtv :/raid #museun\r\n")
}

#[test]
fn encode_slow() {
    test_encode(|enc| enc.slow(Some(42)), "PRIVMSG jtv :/slow 42\r\n");
    test_encode(|enc| enc.slow(42), "PRIVMSG jtv :/slow 42\r\n");
    test_encode(|enc| enc.slow(None), "PRIVMSG jtv :/slow 120\r\n");
}

#[test]
fn encode_slowoff() {
    test_encode(|enc| enc.slow_off(), "PRIVMSG jtv :/slowoff\r\n")
}

#[test]
fn encode_subscribers() {
    test_encode(|enc| enc.subscribers(), "PRIVMSG jtv :/subscribers\r\n")
}

#[test]
fn encode_subscribersoff() {
    test_encode(
        |enc| enc.subscribers_off(),
        "PRIVMSG jtv :/subscribersoff\r\n",
    )
}

#[test]
fn encode_timeout() {
    test_encode(
        |enc| enc.timeout("museun", None, None),
        "PRIVMSG jtv :/timeout museun\r\n",
    );
    test_encode(
        |enc| enc.timeout("museun", Some("1d2h"), None),
        "PRIVMSG jtv :/timeout museun 1d2h\r\n",
    );
    test_encode(
        |enc| enc.timeout("museun", None, Some("spamming")),
        "PRIVMSG jtv :/timeout museun spamming\r\n",
    );
    test_encode(
        |enc| enc.timeout("museun", Some("1d2h"), Some("spamming")),
        "PRIVMSG jtv :/timeout museun 1d2h spamming\r\n",
    );
}

#[test]
fn encode_unban() {
    test_encode(|enc| enc.unban("museun"), "PRIVMSG jtv :/unban museun\r\n")
}

#[test]
fn encode_unhost() {
    test_encode(|enc| enc.unhost(), "PRIVMSG jtv :/unhost\r\n")
}

#[test]
fn encode_unmod() {
    test_encode(|enc| enc.unmod("museun"), "PRIVMSG jtv :/unmod museun\r\n")
}

#[test]
fn encode_unraid() {
    test_encode(|enc| enc.unraid(), "PRIVMSG jtv :/unraid\r\n")
}

#[test]
fn encode_untimeout() {
    test_encode(
        |enc| enc.untimeout("museun"),
        "PRIVMSG jtv :/untimeout museun\r\n",
    )
}

#[test]
fn encode_unvip() {
    test_encode(|enc| enc.unvip("museun"), "PRIVMSG jtv :/unvip museun\r\n")
}

#[test]
fn encode_vip() {
    test_encode(|enc| enc.vip("museun"), "PRIVMSG jtv :/vip museun\r\n")
}

#[test]
fn encode_vips() {
    test_encode(|enc| enc.vips(), "PRIVMSG jtv :/vips\r\n")
}

#[test]
fn encode_whisper() {
    test_encode(
        |enc| enc.whisper("museun", "hello world"),
        "PRIVMSG jtv :/w museun hello world\r\n",
    )
}
