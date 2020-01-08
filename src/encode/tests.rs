use super::*;

async fn test_encode<E: Encodable>(msg: E, expected: impl AsRef<str>) {
    let mut out = vec![];
    encode(&msg, &mut out).await.unwrap();
    assert_eq!(std::str::from_utf8(&out).unwrap(), expected.as_ref());
}

#[tokio::test]
async fn encode_raw() {
    test_encode(
        raw("PRIVMSG #test :this is a test"),
        "PRIVMSG #test :this is a test\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_pong() {
    test_encode(pong("123456789"), "PONG :123456789\r\n").await;
}

#[tokio::test]
async fn encode_ping() {
    test_encode(ping("123456789"), "PING 123456789\r\n").await;
}

#[tokio::test]
async fn encode_join() {
    test_encode(join("#museun"), "JOIN #museun\r\n").await;
}

#[tokio::test]
async fn encode_part() {
    test_encode(part("#museun"), "PART #museun\r\n").await;
}

#[tokio::test]
async fn encode_privmsg() {
    test_encode(
        privmsg("#museun", "this is a test of a line"),
        "PRIVMSG #museun :this is a test of a line\r\n",
    )
    .await;

    test_encode(
        privmsg("#museun", &"foo ".repeat(500)),
        format!("PRIVMSG #museun :{}\r\n", &"foo ".repeat(500)),
    )
    .await;
}

#[tokio::test]
async fn encode_ban() {
    test_encode(ban("museun", None), "PRIVMSG jtv :/ban museun\r\n").await
}

#[tokio::test]
async fn encode_clear() {
    test_encode(clear(), "PRIVMSG jtv :/clear\r\n").await
}

#[tokio::test]
async fn encode_color() {
    let blue = "blue".parse().unwrap();
    test_encode(color(blue), format!("PRIVMSG jtv :/color {}\r\n", blue)).await
}

#[tokio::test]
async fn encode_command() {
    test_encode(command("/testing"), "PRIVMSG jtv :/testing\r\n").await
}

#[tokio::test]
async fn encode_commercial() {
    test_encode(commercial(None), "PRIVMSG jtv :/commercial\r\n").await;
    test_encode(commercial(10), "PRIVMSG jtv :/commercial 10\r\n").await;
    test_encode(commercial(Some(10)), "PRIVMSG jtv :/commercial 10\r\n").await;
}

#[tokio::test]
async fn encode_disconnect() {
    test_encode(disconnect(), "PRIVMSG jtv :/disconnect\r\n").await
}

#[tokio::test]
async fn encode_emoteonly() {
    test_encode(emote_only(), "PRIVMSG jtv :/emoteonly\r\n").await
}

#[tokio::test]
async fn encode_emoteonlyoff() {
    test_encode(emote_only_off(), "PRIVMSG jtv :/emoteonlyoff\r\n").await
}

#[tokio::test]
async fn encode_followers() {
    test_encode(followers("1 week"), "PRIVMSG jtv :/followers 1 week\r\n").await
}

#[tokio::test]
async fn encode_followersoff() {
    test_encode(followers_off(), "PRIVMSG jtv :/followersoff\r\n").await
}

#[tokio::test]
async fn encode_help() {
    test_encode(help(), "PRIVMSG jtv :/help\r\n").await
}

#[tokio::test]
async fn encode_host() {
    test_encode(host("#museun"), "PRIVMSG jtv :/host #museun\r\n").await
}

#[tokio::test]
async fn encode_marker() {
    test_encode(
        marker(Some("this is an example")),
        "PRIVMSG jtv :/marker this is an example\r\n",
    )
    .await;
    test_encode(
        marker("this is an example"),
        "PRIVMSG jtv :/marker this is an example\r\n",
    )
    .await;
    test_encode(marker(None), "PRIVMSG jtv :/marker\r\n").await;
}

#[tokio::test]
async fn encode_me() {
    test_encode(
        me("#museun", "some emote"),
        "PRIVMSG #museun :/me some emote\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_give_mod() {
    test_encode(give_mod("#museun"), "PRIVMSG jtv :/mod #museun\r\n").await
}

#[tokio::test]
async fn encode_mods() {
    test_encode(mods(), "PRIVMSG jtv :/mods\r\n").await
}

#[tokio::test]
async fn encode_r9kbeta() {
    test_encode(r9k_beta(), "PRIVMSG jtv :/r9kbeta\r\n").await
}

#[tokio::test]
async fn encode_r9kbetaoff() {
    test_encode(r9k_beta_off(), "PRIVMSG jtv :/r9kbetaoff\r\n").await
}

#[tokio::test]
async fn encode_raid() {
    test_encode(raid("#museun"), "PRIVMSG jtv :/raid #museun\r\n").await
}

#[tokio::test]
async fn encode_slow() {
    test_encode(slow(Some(42)), "PRIVMSG jtv :/slow 42\r\n").await;
    test_encode(slow(42), "PRIVMSG jtv :/slow 42\r\n").await;
    test_encode(slow(None), "PRIVMSG jtv :/slow 120\r\n").await;
}

#[tokio::test]
async fn encode_slowoff() {
    test_encode(slow_off(), "PRIVMSG jtv :/slowoff\r\n").await
}

#[tokio::test]
async fn encode_subscribers() {
    test_encode(subscribers(), "PRIVMSG jtv :/subscribers\r\n").await
}

#[tokio::test]
async fn encode_subscribersoff() {
    test_encode(subscribers_off(), "PRIVMSG jtv :/subscribersoff\r\n").await
}

#[tokio::test]
async fn encode_timeout() {
    test_encode(
        timeout("museun", None, None),
        "PRIVMSG jtv :/timeout museun\r\n",
    )
    .await;
    test_encode(
        timeout("museun", Some("1d2h"), None),
        "PRIVMSG jtv :/timeout museun 1d2h\r\n",
    )
    .await;
    test_encode(
        timeout("museun", None, Some("spamming")),
        "PRIVMSG jtv :/timeout museun spamming\r\n",
    )
    .await;
    test_encode(
        timeout("museun", Some("1d2h"), Some("spamming")),
        "PRIVMSG jtv :/timeout museun 1d2h spamming\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_unban() {
    test_encode(unban("museun"), "PRIVMSG jtv :/unban museun\r\n").await
}

#[tokio::test]
async fn encode_unhost() {
    test_encode(unhost(), "PRIVMSG jtv :/unhost\r\n").await
}

#[tokio::test]
async fn encode_unmod() {
    test_encode(unmod("museun"), "PRIVMSG jtv :/unmod museun\r\n").await
}

#[tokio::test]
async fn encode_unraid() {
    test_encode(unraid(), "PRIVMSG jtv :/unraid\r\n").await
}

#[tokio::test]
async fn encode_untimeout() {
    test_encode(untimeout("museun"), "PRIVMSG jtv :/untimeout museun\r\n").await
}

#[tokio::test]
async fn encode_unvip() {
    test_encode(unvip("museun"), "PRIVMSG jtv :/unvip museun\r\n").await
}

#[tokio::test]
async fn encode_vip() {
    test_encode(vip("museun"), "PRIVMSG jtv :/vip museun\r\n").await
}

#[tokio::test]
async fn encode_vips() {
    test_encode(vips(), "PRIVMSG jtv :/vips\r\n").await
}

#[tokio::test]
async fn encode_whisper() {
    test_encode(
        whisper("museun", "hello world"),
        "PRIVMSG jtv :/w museun hello world\r\n",
    )
    .await
}
