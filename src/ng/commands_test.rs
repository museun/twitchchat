use super::commands::*;
use crate::ng::Encodable;

fn test_encode(enc: impl Encodable, expected: impl for<'a> PartialEq<&'a str> + std::fmt::Debug) {
    let mut data = vec![];
    enc.encode(&mut data).unwrap();
    assert_eq!(expected, std::str::from_utf8(&data).unwrap());
}

#[test]
fn encode_raw() {
    test_encode(
        raw("PRIVMSG #test :this is a test"),
        "PRIVMSG #test :this is a test\r\n",
    );
}

#[test]
fn encode_pong() {
    test_encode(pong("123456789"), "PONG :123456789\r\n");
}

#[test]
fn encode_ping() {
    test_encode(ping("123456789"), "PING 123456789\r\n");
}

#[test]
fn encode_join() {
    test_encode(join("#museun"), "JOIN #museun\r\n");
}

#[test]
fn encode_part() {
    test_encode(part("#museun"), "PART #museun\r\n");
}

#[test]
fn encode_privmsg() {
    test_encode(
        privmsg("#museun", "this is a test of a line"),
        "PRIVMSG #museun :this is a test of a line\r\n",
    );

    test_encode(
        privmsg("#museun", &"foo ".repeat(500)),
        format!("PRIVMSG #museun :{}\r\n", &"foo ".repeat(500)),
    );
}

#[test]
fn encode_ban() {
    test_encode(
        ban("#museun", "museun", None),
        "PRIVMSG #museun :/ban museun\r\n",
    )
}

#[test]
fn encode_clear() {
    test_encode(clear("#museun"), "PRIVMSG #museun :/clear\r\n")
}

#[test]
fn encode_color() {
    let blue: crate::color::Color = "blue".parse().unwrap();
    test_encode(
        color(blue).unwrap(),
        format!("PRIVMSG jtv :/color {}\r\n", blue),
    )
}

#[test]
fn encode_command() {
    test_encode(
        command("#museun", "/testing"),
        "PRIVMSG #museun :/testing\r\n",
    )
}

#[test]
fn encode_commercial() {
    test_encode(
        commercial("#museun", None),
        "PRIVMSG #museun :/commercial\r\n",
    );
    test_encode(
        commercial("#museun", 10),
        "PRIVMSG #museun :/commercial 10\r\n",
    );
    test_encode(
        commercial("#museun", Some(10)),
        "PRIVMSG #museun :/commercial 10\r\n",
    );
}

#[test]
fn encode_disconnect() {
    test_encode(disconnect(), "PRIVMSG jtv :/disconnect\r\n")
}

#[test]
fn encode_emoteonly() {
    test_encode(emote_only("#museun"), "PRIVMSG #museun :/emoteonly\r\n")
}

#[test]
fn encode_emoteonlyoff() {
    test_encode(
        emote_only_off("#museun"),
        "PRIVMSG #museun :/emoteonlyoff\r\n",
    )
}

#[test]
fn encode_followers() {
    test_encode(
        followers("#museun", "1 week"),
        "PRIVMSG #museun :/followers 1 week\r\n",
    )
}

#[test]
fn encode_followersoff() {
    test_encode(
        followers_off("#museun"),
        "PRIVMSG #museun :/followersoff\r\n",
    )
}

#[test]
fn encode_help() {
    test_encode(help("#museun"), "PRIVMSG #museun :/help\r\n")
}

#[test]
fn encode_host() {
    test_encode(
        host("#museun", "#shaken_bot"),
        "PRIVMSG #museun :/host #shaken_bot\r\n",
    )
}

#[test]
fn encode_marker() {
    test_encode(
        marker("#museun", Some("this is an example")),
        "PRIVMSG #museun :/marker this is an example\r\n",
    );
    test_encode(
        marker("#museun", "this is an example"),
        "PRIVMSG #museun :/marker this is an example\r\n",
    );
    test_encode(
        marker("#museun", "a".repeat(200).as_str()),
        format!("PRIVMSG #museun :/marker {}\r\n", "a".repeat(140)),
    );
    test_encode(marker("#museun", None), "PRIVMSG #museun :/marker\r\n");
}

#[test]
fn encode_me() {
    test_encode(
        me("#museun", "some emote"),
        "PRIVMSG #museun :/me some emote\r\n",
    );
}

#[test]
fn encode_give_mod() {
    test_encode(
        give_mod("#museun", "shaken_bot"),
        "PRIVMSG #museun :/mod shaken_bot\r\n",
    )
}

#[test]
fn encode_mods() {
    test_encode(mods("#museun"), "PRIVMSG #museun :/mods\r\n")
}

#[test]
fn encode_r9kbeta() {
    test_encode(r9k_beta("#museun"), "PRIVMSG #museun :/r9kbeta\r\n")
}

#[test]
fn encode_r9kbetaoff() {
    test_encode(r9k_beta_off("#museun"), "PRIVMSG #museun :/r9kbetaoff\r\n")
}

#[test]
fn encode_raid() {
    test_encode(
        raid("#museun", "#museun"),
        "PRIVMSG #museun :/raid #museun\r\n",
    )
}

#[test]
fn encode_slow() {
    test_encode(slow("#museun", Some(42)), "PRIVMSG #museun :/slow 42\r\n");
    test_encode(slow("#museun", 42), "PRIVMSG #museun :/slow 42\r\n");
    test_encode(slow("#museun", None), "PRIVMSG #museun :/slow 120\r\n");
}

#[test]
fn encode_slowoff() {
    test_encode(slow_off("#museun"), "PRIVMSG #museun :/slowoff\r\n")
}

#[test]
fn encode_subscribers() {
    test_encode(subscribers("#museun"), "PRIVMSG #museun :/subscribers\r\n")
}

#[test]
fn encode_subscribersoff() {
    test_encode(
        subscribers_off("#museun"),
        "PRIVMSG #museun :/subscribersoff\r\n",
    )
}

#[test]
fn encode_timeout() {
    test_encode(
        timeout("#museun", "museun", None, None),
        "PRIVMSG #museun :/timeout museun\r\n",
    );
    test_encode(
        timeout("#museun", "museun", Some("1d2h"), None),
        "PRIVMSG #museun :/timeout museun 1d2h\r\n",
    );
    test_encode(
        timeout("#museun", "museun", None, Some("spamming")),
        "PRIVMSG #museun :/timeout museun spamming\r\n",
    );
    test_encode(
        timeout("#museun", "museun", Some("1d2h"), Some("spamming")),
        "PRIVMSG #museun :/timeout museun 1d2h spamming\r\n",
    );
}

#[test]
fn encode_unban() {
    test_encode(
        unban("#museun", "museun"),
        "PRIVMSG #museun :/unban museun\r\n",
    )
}

#[test]
fn encode_unhost() {
    test_encode(unhost("#museun"), "PRIVMSG #museun :/unhost\r\n")
}

#[test]
fn encode_unmod() {
    test_encode(
        unmod("#museun", "museun"),
        "PRIVMSG #museun :/unmod museun\r\n",
    )
}

#[test]
fn encode_unraid() {
    test_encode(unraid("#museun"), "PRIVMSG #museun :/unraid\r\n")
}

#[test]
fn encode_untimeout() {
    test_encode(
        untimeout("#museun", "museun"),
        "PRIVMSG #museun :/untimeout museun\r\n",
    )
}

#[test]
fn encode_unvip() {
    test_encode(
        unvip("#museun", "museun"),
        "PRIVMSG #museun :/unvip museun\r\n",
    )
}

#[test]
fn encode_vip() {
    test_encode(vip("#museun", "museun"), "PRIVMSG #museun :/vip museun\r\n")
}

#[test]
fn encode_vips() {
    test_encode(vips("#museun"), "PRIVMSG #museun :/vips\r\n")
}

#[test]
fn encode_whisper() {
    test_encode(
        whisper("museun", "hello world"),
        "PRIVMSG jtv :/w museun hello world\r\n",
    )
}

#[cfg(feature = "serde")]
fn test_serde<'de: 't, 't, T>(enc: T, expected: impl for<'a> PartialEq<&'a str> + std::fmt::Debug)
where
    T: ::serde::Serialize + Encodable,
    T: PartialEq + std::fmt::Debug,
    T: ::serde::Deserialize<'de> + 't,
{
    let json = serde_json::to_string_pretty(&enc).unwrap();

    #[derive(Debug, PartialEq, ::serde::Deserialize)]
    struct Wrapper {
        raw: String,
    }

    let wrapper: Wrapper = serde_json::from_str(&json).unwrap();
    assert_eq!(expected, &*wrapper.raw);

    // said json doesn't live for long enough
    // okay.
    let whatever: &'static str = Box::leak(json.into_boxed_str());

    let out = serde_json::from_str::<T>(&whatever).unwrap();
    assert_eq!(out, enc);
}

#[test]
#[cfg(feature = "serde")]
fn raw_serde() {
    test_serde(
        raw("PRIVMSG #test :this is a test"),
        "PRIVMSG #test :this is a test\r\n",
    );
}

#[test]
#[cfg(feature = "serde")]
fn pong_serde() {
    test_serde(pong("123456789"), "PONG :123456789\r\n");
}

#[test]
#[cfg(feature = "serde")]
fn ping_serde() {
    test_serde(ping("123456789"), "PING 123456789\r\n");
}

#[test]
#[cfg(feature = "serde")]
fn join_serde() {
    test_serde(join("#museun"), "JOIN #museun\r\n");
}

#[test]
#[cfg(feature = "serde")]
fn part_serde() {
    test_serde(part("#museun"), "PART #museun\r\n");
}

#[test]
#[cfg(feature = "serde")]
fn privmsg_serde() {
    test_serde(
        privmsg("#museun", "this is a test of a line"),
        "PRIVMSG #museun :this is a test of a line\r\n",
    );

    test_serde(
        privmsg("#museun", &"foo ".repeat(500)),
        format!("PRIVMSG #museun :{}\r\n", &"foo ".repeat(500)),
    );
}

#[test]
#[cfg(feature = "serde")]
fn ban_serde() {
    test_serde(
        ban("#museun", "museun", None),
        "PRIVMSG #museun :/ban museun\r\n",
    )
}

#[test]
#[cfg(feature = "serde")]
fn clear_serde() {
    test_serde(clear("#museun"), "PRIVMSG #museun :/clear\r\n")
}

#[test]
#[cfg(feature = "serde")]
fn color_serde() {
    let blue: crate::color::Color = "blue".parse().unwrap();
    test_serde(
        color(blue).unwrap(),
        format!("PRIVMSG jtv :/color {}\r\n", blue),
    )
}

#[test]
#[cfg(feature = "serde")]
fn command_serde() {
    test_serde(
        command("#museun", "/testing"),
        "PRIVMSG #museun :/testing\r\n",
    )
}

#[test]
#[cfg(feature = "serde")]
fn commercial_serde() {
    test_serde(
        commercial("#museun", None),
        "PRIVMSG #museun :/commercial\r\n",
    );
    test_serde(
        commercial("#museun", 10),
        "PRIVMSG #museun :/commercial 10\r\n",
    );
    test_serde(
        commercial("#museun", Some(10)),
        "PRIVMSG #museun :/commercial 10\r\n",
    );
}

#[test]
#[cfg(feature = "serde")]
fn disconnect_serde() {
    test_serde(disconnect(), "PRIVMSG jtv :/disconnect\r\n")
}

#[test]
#[cfg(feature = "serde")]
fn emoteonly_serde() {
    test_serde(emote_only("#museun"), "PRIVMSG #museun :/emoteonly\r\n")
}

#[test]
#[cfg(feature = "serde")]
fn emoteonlyoff_serde() {
    test_serde(
        emote_only_off("#museun"),
        "PRIVMSG #museun :/emoteonlyoff\r\n",
    )
}

#[test]
#[cfg(feature = "serde")]
fn followers_serde() {
    test_serde(
        followers("#museun", "1 week"),
        "PRIVMSG #museun :/followers 1 week\r\n",
    )
}

#[test]
#[cfg(feature = "serde")]
fn followersoff_serde() {
    test_serde(
        followers_off("#museun"),
        "PRIVMSG #museun :/followersoff\r\n",
    )
}

#[test]
#[cfg(feature = "serde")]
fn help_serde() {
    test_serde(help("#museun"), "PRIVMSG #museun :/help\r\n")
}

#[test]
#[cfg(feature = "serde")]
fn host_serde() {
    test_serde(
        host("#museun", "#shaken_bot"),
        "PRIVMSG #museun :/host #shaken_bot\r\n",
    )
}

#[test]
#[cfg(feature = "serde")]
fn marker_serde() {
    test_serde(
        marker("#museun", Some("this is an example")),
        "PRIVMSG #museun :/marker this is an example\r\n",
    );
    test_serde(
        marker("#museun", "this is an example"),
        "PRIVMSG #museun :/marker this is an example\r\n",
    );
    test_serde(
        marker("#museun", "a".repeat(200).as_str()),
        format!("PRIVMSG #museun :/marker {}\r\n", "a".repeat(140)),
    );
    test_serde(marker("#museun", None), "PRIVMSG #museun :/marker\r\n");
}

#[test]
#[cfg(feature = "serde")]
fn me_serde() {
    test_serde(
        me("#museun", "some emote"),
        "PRIVMSG #museun :/me some emote\r\n",
    );
}

#[test]
#[cfg(feature = "serde")]
fn give_mod_serde() {
    test_serde(
        give_mod("#museun", "shaken_bot"),
        "PRIVMSG #museun :/mod shaken_bot\r\n",
    )
}

#[test]
#[cfg(feature = "serde")]
fn mods_serde() {
    test_serde(mods("#museun"), "PRIVMSG #museun :/mods\r\n")
}

#[test]
#[cfg(feature = "serde")]
fn r9kbeta_serde() {
    test_serde(r9k_beta("#museun"), "PRIVMSG #museun :/r9kbeta\r\n")
}

#[test]
#[cfg(feature = "serde")]
fn r9kbetaoff_serde() {
    test_serde(r9k_beta_off("#museun"), "PRIVMSG #museun :/r9kbetaoff\r\n")
}

#[test]
#[cfg(feature = "serde")]
fn raid_serde() {
    test_serde(
        raid("#museun", "#museun"),
        "PRIVMSG #museun :/raid #museun\r\n",
    )
}

#[test]
#[cfg(feature = "serde")]
fn slow_serde() {
    test_serde(slow("#museun", Some(42)), "PRIVMSG #museun :/slow 42\r\n");
    test_serde(slow("#museun", 42), "PRIVMSG #museun :/slow 42\r\n");
    test_serde(slow("#museun", None), "PRIVMSG #museun :/slow 120\r\n");
}

#[test]
#[cfg(feature = "serde")]
fn slowoff_serde() {
    test_serde(slow_off("#museun"), "PRIVMSG #museun :/slowoff\r\n")
}

#[test]
#[cfg(feature = "serde")]
fn subscribers_serde() {
    test_serde(subscribers("#museun"), "PRIVMSG #museun :/subscribers\r\n")
}

#[test]
#[cfg(feature = "serde")]
fn subscribersoff_serde() {
    test_serde(
        subscribers_off("#museun"),
        "PRIVMSG #museun :/subscribersoff\r\n",
    )
}

#[test]
#[cfg(feature = "serde")]
fn timeout_serde() {
    test_serde(
        timeout("#museun", "museun", None, None),
        "PRIVMSG #museun :/timeout museun\r\n",
    );
    test_serde(
        timeout("#museun", "museun", Some("1d2h"), None),
        "PRIVMSG #museun :/timeout museun 1d2h\r\n",
    );
    test_serde(
        timeout("#museun", "museun", None, Some("spamming")),
        "PRIVMSG #museun :/timeout museun spamming\r\n",
    );
    test_serde(
        timeout("#museun", "museun", Some("1d2h"), Some("spamming")),
        "PRIVMSG #museun :/timeout museun 1d2h spamming\r\n",
    );
}

#[test]
#[cfg(feature = "serde")]
fn unban_serde() {
    test_serde(
        unban("#museun", "museun"),
        "PRIVMSG #museun :/unban museun\r\n",
    )
}

#[test]
#[cfg(feature = "serde")]
fn unhost_serde() {
    test_serde(unhost("#museun"), "PRIVMSG #museun :/unhost\r\n")
}

#[test]
#[cfg(feature = "serde")]
fn unmod_serde() {
    test_serde(
        unmod("#museun", "museun"),
        "PRIVMSG #museun :/unmod museun\r\n",
    )
}

#[test]
#[cfg(feature = "serde")]
fn unraid_serde() {
    test_serde(unraid("#museun"), "PRIVMSG #museun :/unraid\r\n")
}

#[test]
#[cfg(feature = "serde")]
fn untimeout_serde() {
    test_serde(
        untimeout("#museun", "museun"),
        "PRIVMSG #museun :/untimeout museun\r\n",
    )
}

#[test]
#[cfg(feature = "serde")]
fn unvip_serde() {
    test_serde(
        unvip("#museun", "museun"),
        "PRIVMSG #museun :/unvip museun\r\n",
    )
}

#[test]
#[cfg(feature = "serde")]
fn vip_serde() {
    test_serde(vip("#museun", "museun"), "PRIVMSG #museun :/vip museun\r\n")
}

#[test]
#[cfg(feature = "serde")]
fn vips_serde() {
    test_serde(vips("#museun"), "PRIVMSG #museun :/vips\r\n")
}

#[test]
#[cfg(feature = "serde")]
fn whisper_serde() {
    test_serde(
        whisper("museun", "hello world"),
        "PRIVMSG jtv :/w museun hello world\r\n",
    )
}
