use std::io::prelude::*;

/// Encode this message to the buffer
///
/// See available messages in the [encode](./index.html#structs) module
pub trait Encodable: private::Sealed {
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

macro_rules! export_encode {
    ($($ident:ident)*) => {
        $( mod $ident; pub use $ident::*; )*
    };
}

export_encode! {
    raw
    ping
    pong
    join
    part
    privmsg
}

#[cfg(test)]
mod tests;
