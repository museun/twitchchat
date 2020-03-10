/** A trait for parsing messages

# Example
```rust
# use twitchchat::*;
# use twitchchat::messages::*;
# use std::borrow::Cow;
let input = ":test!test@test JOIN #museun\r\n";
// decode and decode_one will parse a `Message` from a string
let message: decode::Message<'_> = decode::decode(&input).next().unwrap().unwrap();

// which can be used to parse into a specific message
let join: Join<'_> = Join::parse(&message).unwrap();
assert_eq!(join, Join {
    channel: Cow::Borrowed("#museun"),
    name: Cow::Borrowed("test")
});
```
*/
pub trait Parse<T>: Sized + private::ParseSealed<T> {
    /// Tries to parse the input as this message
    fn parse(input: T) -> Result<Self, crate::messages::InvalidMessage>;
}

mod private {
    pub trait ParseSealed<E> {}
    impl<T: crate::Parse<E>, E: Sized> ParseSealed<E> for T {}
}
