use std::io::{Result, Write};

/// Mirror all writes to a
/// [std::io::Write](https://doc.rust-lang.org/std/io/trait.Write.html) to another
/// [std::io::Write](https://doc.rust-lang.org/std/io/trait.Write.html)
///
/// This is useful if you want to log all writes from a `Write`er to some file (the `Write`r)
pub struct TeeWriter<L, R> {
    write: L,
    output: R,
}

impl<L, R> TeeWriter<L, R> {
    /// Create a new TeeWriter from two Write impls.
    pub fn new(write: L, output: R) -> Self {
        Self { write, output }
    }

    /// Moves the wrapped Read and Write out
    pub fn into_inner(self) -> (L, R) {
        (self.write, self.output)
    }

    /// Borrows the output type
    pub fn borrow_output(&self) -> &R {
        &self.output
    }

    /// Borrows the output type mutably
    pub fn borrow_output_mut(&mut self) -> &mut R {
        &mut self.output
    }
}

impl<L: Write, R: Write> Write for TeeWriter<L, R> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let n = self.write.write(buf)?;
        let _ = self.output.write(&buf[..n])?;
        Ok(n)
    }

    fn flush(&mut self) -> Result<()> {
        self.write.flush()
    }
}
