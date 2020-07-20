use crate::UserConfig;
use futures_lite::{AsyncWrite, AsyncWriteExt};

/**
Write the provided [`UserConfig`](./struct.UserConfig.html) to the ***async*** writer

# Example
Write it to a TcpStream
```rust
# use twitchchat::*;
# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let config = UserConfig::builder().anonymous().build()?;

let mut writer = vec![]; // or any type that impls AsyncWrite
register(&config, &mut writer).await?;

assert_eq!(
    std::str::from_utf8(&writer)?,
    "PASS justinfan1234\r\nNICK justinfan1234\r\n"
);

# Ok::<_, Box<dyn std::error::Error>>(())
# }).unwrap();
```
*/
pub async fn register<W: ?Sized>(user_config: &UserConfig, writer: &mut W) -> std::io::Result<()>
where
    W: AsyncWrite + Unpin + Send,
{
    let UserConfig {
        name,
        token,
        capabilities,
    } = user_config;

    for cap in capabilities {
        let cap = cap.encode_as_str();
        log::trace!("sending CAP: {}", cap);
        writer.write_all(cap.as_bytes()).await?;
        writer.write_all(b"\r\n").await?;
    }

    log::trace!(
        "sending PASS: {} (len={})",
        "*".repeat(token.len()),
        token.len()
    );
    log::trace!("sending NICK: {}", name);
    writer
        .write_all(format!("PASS {}\r\nNICK {}\r\n", token, name).as_bytes())
        .await?;

    log::trace!("flushing initial handshake");
    writer.flush().await?;
    Ok(())
}

/**
Write the provided `name` and `token` to the ***async*** writer

This enables all of the [Capabilities]

[Capabilities]: ./enum.Capability.html

# Example
```rust
# use twitchchat::*;
# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let mut writer = vec![]; // or any type that impls AsyncWrite
register_easy("museun", "oauth:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", &mut writer).await?;

let lines = std::str::from_utf8(&writer)?.split_terminator("\r\n").collect::<Vec<_>>();
assert_eq!(
    lines,
    vec![
        "CAP REQ :twitch.tv/membership",
        "CAP REQ :twitch.tv/tags",
        "CAP REQ :twitch.tv/commands",
        "PASS oauth:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "NICK museun",
    ],
);

# Ok::<_, Box<dyn std::error::Error>>(())
# }).unwrap();
```
*/
pub async fn register_easy<W: ?Sized>(
    name: &str,
    token: &str,
    writer: &mut W,
) -> std::io::Result<()>
where
    W: AsyncWrite + Unpin + Send,
{
    let config = crate::simple_user_config(name, token).unwrap();
    register(&config, writer).await
}
