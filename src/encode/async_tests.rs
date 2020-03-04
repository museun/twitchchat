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
            enc.ban("museun", None).await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/ban museun\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_clear() {
    test_encode(
        |mut enc| async move {
            enc.clear().await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/clear\r\n",
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
            enc.command("/testing").await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/testing\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_commercial() {
    test_encode(
        |mut enc| async move {
            enc.commercial(None).await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/commercial\r\n",
    )
    .await;
    test_encode(
        |mut enc| async move {
            enc.commercial(10).await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/commercial 10\r\n",
    )
    .await;
    test_encode(
        |mut enc| async move {
            enc.commercial(Some(10)).await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/commercial 10\r\n",
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
            enc.emote_only().await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/emoteonly\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_emoteonlyoff() {
    test_encode(
        |mut enc| async move {
            enc.emote_only_off().await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/emoteonlyoff\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_followers() {
    test_encode(
        |mut enc| async move {
            enc.followers("1 week").await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/followers 1 week\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_followersoff() {
    test_encode(
        |mut enc| async move {
            enc.followers_off().await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/followersoff\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_help() {
    test_encode(
        |mut enc| async move {
            enc.help().await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/help\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_host() {
    test_encode(
        |mut enc| async move {
            enc.host("#museun").await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/host #museun\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_marker() {
    test_encode(
        |mut enc| async move {
            enc.marker(Some("this is an example")).await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/marker this is an example\r\n",
    )
    .await;
    test_encode(
        |mut enc| async move {
            enc.marker("this is an example").await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/marker this is an example\r\n",
    )
    .await;
    test_encode(
        |mut enc| async move {
            enc.marker(None).await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/marker\r\n",
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
            enc.give_mod("#museun").await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/mod #museun\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_mods() {
    test_encode(
        |mut enc| async move {
            enc.mods().await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/mods\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_r9kbeta() {
    test_encode(
        |mut enc| async move {
            enc.r9k_beta().await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/r9kbeta\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_r9kbetaoff() {
    test_encode(
        |mut enc| async move {
            enc.r9k_beta_off().await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/r9kbetaoff\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_raid() {
    test_encode(
        |mut enc| async move {
            enc.raid("#museun").await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/raid #museun\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_slow() {
    test_encode(
        |mut enc| async move {
            enc.slow(Some(42)).await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/slow 42\r\n",
    )
    .await;
    test_encode(
        |mut enc| async move {
            enc.slow(42).await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/slow 42\r\n",
    )
    .await;
    test_encode(
        |mut enc| async move {
            enc.slow(None).await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/slow 120\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_slowoff() {
    test_encode(
        |mut enc| async move {
            enc.slow_off().await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/slowoff\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_subscribers() {
    test_encode(
        |mut enc| async move {
            enc.subscribers().await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/subscribers\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_subscribersoff() {
    test_encode(
        |mut enc| async move {
            enc.subscribers_off().await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/subscribersoff\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_timeout() {
    test_encode(
        |mut enc| async move {
            enc.timeout("museun", None, None).await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/timeout museun\r\n",
    )
    .await;
    test_encode(
        |mut enc| async move {
            enc.timeout("museun", Some("1d2h"), None).await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/timeout museun 1d2h\r\n",
    )
    .await;
    test_encode(
        |mut enc| async move {
            enc.timeout("museun", None, Some("spamming")).await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/timeout museun spamming\r\n",
    )
    .await;
    test_encode(
        |mut enc| async move {
            enc.timeout("museun", Some("1d2h"), Some("spamming"))
                .await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/timeout museun 1d2h spamming\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_unban() {
    test_encode(
        |mut enc| async move {
            enc.unban("museun").await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/unban museun\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_unhost() {
    test_encode(
        |mut enc| async move {
            enc.unhost().await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/unhost\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_unmod() {
    test_encode(
        |mut enc| async move {
            enc.unmod("museun").await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/unmod museun\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_unraid() {
    test_encode(
        |mut enc| async move {
            enc.unraid().await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/unraid\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_untimeout() {
    test_encode(
        |mut enc| async move {
            enc.untimeout("museun").await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/untimeout museun\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_unvip() {
    test_encode(
        |mut enc| async move {
            enc.unvip("museun").await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/unvip museun\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_vip() {
    test_encode(
        |mut enc| async move {
            enc.vip("museun").await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/vip museun\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_vips() {
    test_encode(
        |mut enc| async move {
            enc.vips().await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/vips\r\n",
    )
    .await;
}

#[tokio::test]
async fn encode_whisper() {
    test_encode(
        |mut enc| async move {
            enc.whisper("museun", "hello world").await?;
            Ok(enc)
        },
        "PRIVMSG jtv :/w museun hello world\r\n",
    )
    .await;
}
