use crate::UserConfig;
use tokio::prelude::*;

/**
Write the provided `UserConfig` to the ***async*** writer

# Example
```rust
# use twitchchat::*;
# tokio::runtime::Runtime::new().unwrap().block_on(async move {
let config = UserConfig::builder().anonymous().build().unwrap();

let mut writer = vec![];
register(&config, &mut writer).await.unwrap();

assert_eq!(
    std::str::from_utf8(&writer).unwrap(),
    "PASS justinfan1234\r\nNICK justinfan1234\r\n"
);
# });
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
