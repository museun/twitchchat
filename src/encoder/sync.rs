use crate::Encodable;
use std::io::{Result as IoResult, Write};

/// A synchronous encoder
pub struct Encoder<W> {
    writer: W,
}

impl<W> std::fmt::Debug for Encoder<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Encoder").finish()
    }
}

impl<W> Encoder<W>
where
    W: Write,
{
    /// Create a new Encoder over this `std::io::Write` instance
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Get the inner `std::io::Write` instance out
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Encode this `Encodable` message to the writer and flushes it.
    pub fn encode<M>(&mut self, msg: M) -> IoResult<()>
    where
        M: Encodable,
    {
        msg.encode(&mut self.writer)?;
        self.writer.flush()
    }
}

impl<W> Clone for Encoder<W>
where
    W: Clone,
{
    fn clone(&self) -> Self {
        Self {
            writer: self.writer.clone(),
        }
    }
}

impl<W> Write for Encoder<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        self.writer.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::join;

    #[test]
    fn encoder() {
        let mut encoder = Encoder::new(vec![]);

        encoder.encode(join("#museun")).unwrap();
        encoder.encode(join("#shaken_bot")).unwrap();

        // using into_inner here instead of &mut borrowing the vec and dropping the encoder
        let out = encoder.into_inner();
        let s = std::str::from_utf8(&out).unwrap();
        assert_eq!(s, "JOIN #museun\r\nJOIN #shaken_bot\r\n");
    }

    #[test]
    fn encodable_builtin() {
        fn check<T>(input: &T)
        where
            T: Encodable + AsRef<[u8]> + ?Sized,
        {
            let mut output = vec![];
            let mut encoder = Encoder::new(&mut output);
            encoder.encode(input).unwrap();
            assert_eq!(output, input.as_ref());
        }

        let input = "hello world\r\n";
        check(&input);
        check(&input.to_string());
        check(&input.as_bytes());
        check(&input.as_bytes().to_vec());
    }
}
