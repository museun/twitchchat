/*!
This module allows you to mirror read/writes to another read/write (like POSIX tee)

# Examples
## TeeReader
```rust
# use std::io::prelude::*;
// make a new reader
let reader = std::io::Cursor::new(vec![1,2,3]);
let mut tee = tee::TeeReader::new(
    reader,
    vec![], // vec implements write
    false   // we don't care about flushing here
);

// read all of the elements from the cursor into this vec
// each 'read' call will be written to the wrapped writer
let mut results = vec![];
assert_eq!(tee.read_to_end(&mut results).expect("read"), 3);

// consume the tee, returning the reader and the mirroring writer
let (_read, output) = tee.into_inner();
assert_eq!(results, output);
```

## TeeWriter
```rust
# use std::io::prelude::*;
let writer = vec![];
let mut tee = tee::TeeWriter::new(writer, vec![]);
for i in 1..=3 {
    let _ = tee.write_all(&[i]);
}
// we can borrow the output writer
assert_eq!(tee.borrow_output(), &[1,2,3]);

// consume the tee, returning the writer and its tee output
let (left, output) = tee.into_inner();
assert_eq!(left, output);
assert_eq!(output, &[1,2,3]);
```
*/

mod read;
#[doc(inline)]
pub use read::TeeReader;

/// Mirror all writes from an
/// [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html) to
/// another
/// [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html)
mod write;
#[doc(inline)]
pub use write::TeeWriter;
