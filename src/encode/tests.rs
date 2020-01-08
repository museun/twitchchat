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
#[ignore]
async fn encode_ban() {}

#[tokio::test]
#[ignore]
async fn encode_clear() {}

#[tokio::test]
#[ignore]
async fn encode_color() {}

#[tokio::test]
#[ignore]
async fn encode_command() {}

#[tokio::test]
#[ignore]
async fn encode_commercial() {}

#[tokio::test]
#[ignore]
async fn encode_delete() {}

#[tokio::test]
#[ignore]
async fn encode_disconnect() {}

#[tokio::test]
#[ignore]
async fn encode_emoteonly() {}

#[tokio::test]
#[ignore]
async fn encode_emoteonlyoff() {}

#[tokio::test]
#[ignore]
async fn encode_followers() {}

#[tokio::test]
#[ignore]
async fn encode_followersoff() {}

#[tokio::test]
#[ignore]
async fn encode_help() {}

#[tokio::test]
#[ignore]
async fn encode_host() {}

#[tokio::test]
#[ignore]
async fn encode_marker() {}

#[tokio::test]
#[ignore]
async fn encode_me() {}

#[tokio::test]
#[ignore]
async fn encode_give_mod() {}

#[tokio::test]
#[ignore]
async fn encode_mods() {}

#[tokio::test]
#[ignore]
async fn encode_op() {}

#[tokio::test]
#[ignore]
async fn encode_r9kbeta() {}

#[tokio::test]
#[ignore]
async fn encode_r9kbetaoff() {}

#[tokio::test]
#[ignore]
async fn encode_raid() {}

#[tokio::test]
#[ignore]
async fn encode_send() {}

#[tokio::test]
#[ignore]
async fn encode_slow() {}

#[tokio::test]
#[ignore]
async fn encode_slowoff() {}

#[tokio::test]
#[ignore]
async fn encode_subscribers() {}

#[tokio::test]
#[ignore]
async fn encode_subscribersoff() {}

#[tokio::test]
#[ignore]
async fn encode_timeout() {}

#[tokio::test]
#[ignore]
async fn encode_unban() {}

#[tokio::test]
#[ignore]
async fn encode_unhost() {}

#[tokio::test]
#[ignore]
async fn encode_unmod() {}

#[tokio::test]
#[ignore]
async fn encode_unraid() {}

#[tokio::test]
#[ignore]
async fn encode_untimeout() {}

#[tokio::test]
#[ignore]
async fn encode_unvip() {}

#[tokio::test]
#[ignore]
async fn encode_vip() {}

#[tokio::test]
#[ignore]
async fn encode_vips() {}

#[tokio::test]
#[ignore]
async fn encode_whisper() {}
