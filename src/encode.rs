use std::io::prelude::*;

/// Encode this message to the buffer
///
/// See available messages in the [encode](./index.html#structs) module
pub trait Encodable: private::Sealed {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()>;
}

cfg_async! {
    /// Encode the provided message to the [tokio::io::AsyncWrite][AsyncWrite]
    ///
    /// [AsyncWrite]: https://docs.rs/tokio/0.2.6/tokio/io/trait.AsyncWrite.html
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

/// A Raw message. Used to send a raw message
#[derive(Copy, Clone, Debug)]
pub struct Raw<'a> {
    raw: &'a str,
}

impl<'a> Encodable for Raw<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(self.raw.as_bytes())?;
        writer.write_all(b"\r\n")
    }
}

/// A Ping message. Used to request a heartbeat
#[derive(Copy, Clone, Debug)]
pub struct Ping<'a> {
    token: &'a str,
}

impl<'a> Encodable for Ping<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(b"PING ")?;
        writer.write_all(self.token.as_bytes())?;
        writer.write_all(b"\r\n")
    }
}

/// A Pong message. Used to respond to a heartbeat
#[derive(Copy, Clone, Debug)]
pub struct Pong<'a> {
    token: &'a str,
}

impl<'a> Encodable for Pong<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(b"PONG :")?;
        writer.write_all(self.token.as_bytes())?;
        writer.write_all(b"\r\n")
    }
}

/// A Join message. Used to join a channel
#[derive(Copy, Clone, Debug)]
pub struct Join<'a> {
    channel: &'a str,
}

impl<'a> Encodable for Join<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(b"JOIN ")?;
        writer.write_all(self.channel.as_bytes())?;
        writer.write_all(b"\r\n")
    }
}

/// A Part message. Used to leave a channel
#[derive(Copy, Clone, Debug)]
pub struct Part<'a> {
    channel: &'a str,
}

impl<'a> Encodable for Part<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(b"PART ")?;
        writer.write_all(self.channel.as_bytes())?;
        writer.write_all(b"\r\n")
    }
}

/// A Privmsg message. Used to send data to a target
#[derive(Copy, Clone, Debug)]
pub struct Privmsg<'a> {
    target: &'a str,
    data: &'a str,
}

impl<'a> Encodable for Privmsg<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(b"PRIVMSG ")?;
        writer.write_all(self.target.as_bytes())?;
        writer.write_all(b" :")?;
        writer.write_all(self.data.as_bytes())?;
        writer.write_all(b"\r\n")
    }
}

/// Send a raw message
pub fn raw(raw: &str) -> Raw<'_> {
    Raw { raw }
}

/// Request a heartbeat
pub fn ping(token: &str) -> Ping<'_> {
    Ping { token }
}

/// Response to a heartbeat
pub fn pong(token: &str) -> Pong<'_> {
    Pong { token }
}

/// Join a channel
pub fn join(channel: &str) -> Join<'_> {
    Join { channel }
}

/// Leave a channel
pub fn part(channel: &str) -> Part<'_> {
    Part { channel }
}

/// Send data to a target
pub fn privmsg<'a>(target: &'a str, data: &'a str) -> Privmsg<'a> {
    Privmsg { target, data }
}

mod private {
    use super::*;
    pub trait Sealed {}
    impl<T> Sealed for T where T: Encodable {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn encode_pong() {
        let mut out = vec![];
        encode(&pong("123456789"), &mut out).await.unwrap();
        assert_eq!(out, b"PONG :123456789\r\n");
    }

    #[tokio::test]
    async fn encode_join() {
        let mut out = vec![];
        encode(&join("#museun"), &mut out).await.unwrap();
        assert_eq!(out, b"JOIN #museun\r\n");
    }

    #[tokio::test]
    async fn encode_part() {
        let mut out = vec![];
        encode(&part("#museun"), &mut out).await.unwrap();
        assert_eq!(out, b"PART #museun\r\n");
    }

    #[tokio::test]
    async fn encode_privmsg() {
        let mut out = vec![];
        encode(&privmsg("#museun", "this is a test of a line"), &mut out)
            .await
            .unwrap();
        assert_eq!(
            out,
            "PRIVMSG #museun :this is a test of a line\r\n".as_bytes()
        );

        let mut out = vec![];
        encode(&privmsg("#museun", &"foo ".repeat(500)), &mut out)
            .await
            .unwrap();
        assert_eq!(
            out,
            format!("PRIVMSG #museun :{}\r\n", &"foo ".repeat(500)).as_bytes()
        );
    }
}
