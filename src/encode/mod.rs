use std::io::prelude::*;

/// Encode this message to the buffer
///
/// See available messages in the [encode](./index.html#structs) module
pub trait Encodable: private::Sealed {
    /// Encode the provided message to the writer
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()>;
}

mod private {
    use super::*;
    pub trait Sealed {}
    impl<T> Sealed for T where T: Encodable {}
}

cfg_async! {
    /// Encode the provided message to the [tokio::io::AsyncWrite][AsyncWrite]
    ///
    /// [AsyncWrite]: https://docs.rs/tokio/0.2.6/tokio/io/trait.AsyncWrite.html
    ///
    /// # Example
    /// ```rust
    /// # use twitchchat::*;
    /// # tokio::runtime::Runtime::new().unwrap().block_on(async move {
    /// let mut writer = vec![];
    /// let message = encode::join("#museun");
    /// encode(&message, &mut writer).await.unwrap();
    /// assert_eq!(
    ///     std::str::from_utf8(&writer).unwrap(),
    ///     "JOIN #museun\r\n"
    /// );
    /// # });
    /// ```
    pub async fn encode<M: ?Sized, W: ?Sized>(message: &M, writer: &mut W) -> std::io::Result<()>
    where
        M: Encodable,
        W: tokio::io::AsyncWrite + Unpin,
    {
        let mut vec = vec![];
        message.encode(&mut vec)?;

        use tokio::prelude::*;
        writer.write_all(&vec).await?;
        writer.flush().await
    }
}

// TODO get motivated for this
// TODO part 2: remove it and merge this into a non-runtime Writer
export_modules_without_docs! {
    ban
    clear
    color
    command
    commercial
    disconnect
    emoteonly
    emoteonlyoff
    followers
    followersoff
    give_mod
    help
    host
    join
    marker
    me
    mods
    part
    ping
    pong
    privmsg
    r9kbeta
    r9kbetaoff
    raid
    raw
    slow
    slowoff
    subscribers
    subscribersoff
    timeout
    unban
    unhost
    unmod
    unraid
    untimeout
    unvip
    vip
    vips
    whisper
}

#[cfg(all(test, features = "async"))]
mod tests;
