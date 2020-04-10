use super::*;

use futures::prelude::*;

async fn test_encode<F, Fut>(func: F, expected: impl AsRef<str>)
where
    F: FnOnce(AsyncEncoder<Vec<u8>>) -> Fut + Send + 'static,
    Fut: Future<Output = Result<AsyncEncoder<Vec<u8>>, crate::Error>> + Send + 'static,
{
    let encoder = func(Default::default()).await.unwrap();
    assert_eq!(
        std::str::from_utf8(&encoder.into_inner()).unwrap(),
        expected.as_ref()
    );
}

#[tokio::test]
async fn encode_raw() {
    test_encode(
        |mut enc| async move {
            enc.raw("PRIVMSG #test :this is a test").await?;
            Ok(enc)
        },
        "PRIVMSG #test :this is a test\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_pong() {
    test_encode(
        |mut enc| async move {
            enc.pong("123456789").await?;
            Ok(enc)
        },
        "PONG :123456789\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_ping() {
    test_encode(
        |mut enc| async move {
            enc.ping("123456789").await?;
            Ok(enc)
        },
        "PING 123456789\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_join() {
    test_encode(
        |mut enc| async move {
            enc.join("#museun").await?;
            Ok(enc)
        },
        "JOIN #museun\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_part() {
    test_encode(
        |mut enc| async move {
            enc.part("#museun").await?;
            Ok(enc)
        },
        "PART #museun\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_privmsg() {
    test_encode(
        |mut enc| async move {
            enc.privmsg("#museun", "this is a test of a line").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :this is a test of a line\r\n",
    )
    .await;

    test_encode(
        |mut enc| async move {
            enc.privmsg("#museun", &"foo ".repeat(500)).await?;
            Ok(enc)
        },
        format!("PRIVMSG #museun :{}\r\n", &"foo ".repeat(500)),
    )
    .await;
}

#[tokio::test]
async fn encode_ban() {
    test_encode(
        |mut enc| async move {
            enc.ban("#museun", "museun", None).await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/ban museun\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_clear() {
    test_encode(
        |mut enc| async move {
            enc.clear("#museun").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/clear\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_color() {
    let blue = "blue".parse().unwrap();
    test_encode(
        move |mut enc| async move {
            enc.color(blue).await?;
            Ok(enc)
        },
        format!("PRIVMSG jtv :/color {}\r\n", blue),
    )
    .await;
}

#[tokio::test]
async fn encode_command() {
    test_encode(
        |mut enc| async move {
            enc.command("#museun", "/testing").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/testing\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_commercial() {
    test_encode(
        |mut enc| async move {
            enc.commercial("#museun", None).await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/commercial\r\n",
    )
    .await;
    test_encode(
        |mut enc| async move {
            enc.commercial("#museun", 10).await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/commercial 10\r\n",
    )
    .await;
    test_encode(
        |mut enc| async move {
            enc.commercial("#museun", Some(10)).await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/commercial 10\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_disconnect() {
    test_encode(
        |mut enc| async move {
            enc.disconnect().await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/disconnect\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_emoteonly() {
    test_encode(
        |mut enc| async move {
            enc.emote_only("#museun").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/emoteonly\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_emoteonlyoff() {
    test_encode(
        |mut enc| async move {
            enc.emote_only_off("#museun").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/emoteonlyoff\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_followers() {
    test_encode(
        |mut enc| async move {
            enc.followers("#museun", "1 week").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/followers 1 week\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_followersoff() {
    test_encode(
        |mut enc| async move {
            enc.followers_off("#museun").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/followersoff\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_help() {
    test_encode(
        |mut enc| async move {
            enc.help("#museun").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/help\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_host() {
    test_encode(
        |mut enc| async move {
            enc.host("#museun", "#shaken_bot").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/host #shaken_bot\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_marker() {
    test_encode(
        |mut enc| async move {
            enc.marker("#museun", Some("this is an example")).await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/marker this is an example\r\n",
    )
    .await;

    test_encode(
        |mut enc| async move {
            enc.marker("#museun", "this is an example").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/marker this is an example\r\n",
    )
    .await;

    test_encode(
        |mut enc| async move {
            enc.marker("#museun", "a".repeat(200).as_str()).await?;
            Ok(enc)
        },
        format!("PRIVMSG #museun :/marker {}\r\n", "a".repeat(140)),
    )
    .await;

    test_encode(
        |mut enc| async move {
            enc.marker("#museun", None).await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/marker\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_me() {
    test_encode(
        |mut enc| async move {
            enc.me("#museun", "some emote").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/me some emote\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_give_mod() {
    test_encode(
        |mut enc| async move {
            enc.give_mod("#museun", "shaken_bot").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/mod shaken_bot\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_mods() {
    test_encode(
        |mut enc| async move {
            enc.mods("#museun").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/mods\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_r9kbeta() {
    test_encode(
        |mut enc| async move {
            enc.r9k_beta("#museun").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/r9kbeta\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_r9kbetaoff() {
    test_encode(
        |mut enc| async move {
            enc.r9k_beta_off("#museun").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/r9kbetaoff\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_raid() {
    test_encode(
        |mut enc| async move {
            enc.raid("#museun", "#shaken_bot").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/raid #shaken_bot\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_slow() {
    test_encode(
        |mut enc| async move {
            enc.slow("#museun", Some(42)).await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/slow 42\r\n",
    )
    .await;
    test_encode(
        |mut enc| async move {
            enc.slow("#museun", 42).await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/slow 42\r\n",
    )
    .await;
    test_encode(
        |mut enc| async move {
            enc.slow("#museun", None).await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/slow 120\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_slowoff() {
    test_encode(
        |mut enc| async move {
            enc.slow_off("#museun").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/slowoff\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_subscribers() {
    test_encode(
        |mut enc| async move {
            enc.subscribers("#museun").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/subscribers\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_subscribersoff() {
    test_encode(
        |mut enc| async move {
            enc.subscribers_off("#museun").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/subscribersoff\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_timeout() {
    test_encode(
        |mut enc| async move {
            enc.timeout("#museun", "museun", None, None).await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/timeout museun\r\n",
    )
    .await;
    test_encode(
        |mut enc| async move {
            enc.timeout("#museun", "museun", Some("1d2h"), None).await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/timeout museun 1d2h\r\n",
    )
    .await;
    test_encode(
        |mut enc| async move {
            enc.timeout("#museun", "museun", None, Some("spamming"))
                .await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/timeout museun spamming\r\n",
    )
    .await;
    test_encode(
        |mut enc| async move {
            enc.timeout("#museun", "museun", Some("1d2h"), Some("spamming"))
                .await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/timeout museun 1d2h spamming\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_unban() {
    test_encode(
        |mut enc| async move {
            enc.unban("#museun", "museun").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/unban museun\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_unhost() {
    test_encode(
        |mut enc| async move {
            enc.unhost("#museun").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/unhost\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_unmod() {
    test_encode(
        |mut enc| async move {
            enc.unmod("#museun", "museun").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/unmod museun\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_unraid() {
    test_encode(
        |mut enc| async move {
            enc.unraid("#museun").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/unraid\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_untimeout() {
    test_encode(
        |mut enc| async move {
            enc.untimeout("#museun", "museun").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/untimeout museun\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_unvip() {
    test_encode(
        |mut enc| async move {
            enc.unvip("#museun", "museun").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/unvip museun\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_vip() {
    test_encode(
        |mut enc| async move {
            enc.vip("#museun", "museun").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/vip museun\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_vips() {
    test_encode(
        |mut enc| async move {
            enc.vips("#museun").await?;
            Ok(enc)
        },
        "PRIVMSG #museun :/vips\r\n",
    )
    .await;
}
