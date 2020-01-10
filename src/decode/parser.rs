use super::*;

// @tags :prefix cmd args :data\r\n
pub struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    // '@tags '
    pub fn tags(&mut self) -> Tags<&'a str> {
        let input = &self.input[self.pos..];
        if input.starts_with('@') {
            if let Some(pos) = input.find(' ') {
                self.pos += pos + 1;
                return Tags::parse(&input[..pos]).unwrap_or_default();
            }
        }
        Tags::default()
    }

    // ':prefix '
    pub fn prefix(&mut self) -> Option<Prefix<&'a str>> {
        let input = &self.input[self.pos..];
        if !input.starts_with("tmi.twitch.tv") && !input.starts_with(':') {
            return None;
        }
        let pos = input.find(' ')?;
        self.pos += pos + 1;
        Prefix::parse(&input[..pos])
    }

    // 'cmd '
    pub fn command(&mut self) -> &'a str {
        let input = &self.input[self.pos..];
        let pos = input.find(' ').unwrap_or_else(|| input.len());
        self.pos += pos + 1;
        &input[..pos]
    }

    // 'args '
    pub fn args(&mut self) -> &'a str {
        if self.pos > self.input.len() {
            return "";
        }

        let input = &self.input[self.pos..];
        let pos = input.find(':').unwrap_or_else(|| input.len());
        self.pos += pos + 1;
        &input[..pos].trim()
    }

    // ':data'
    pub fn data(&mut self) -> Option<&'a str> {
        self.input.get(self.pos..).filter(|s| !s.is_empty())
    }
}

pub struct ParseIter<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> ParseIter<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }
}

impl<'a> Iterator for ParseIter<'a> {
    type Item = Result<Message<&'a str>>;
    fn next(&mut self) -> Option<Self::Item> {
        const CRLF: &str = "\r\n";
        if self.pos == self.input.len() {
            return None;
        }

        let index = match self.input[self.pos..].find(CRLF) {
            Some(index) => index + CRLF.len() + self.pos,
            None => {
                let err = Err(ParseError::IncompleteMessage { pos: self.pos });
                self.pos = self.input.len(); // so we can bail
                return err.into();
            }
        };

        let pos = std::mem::replace(&mut self.pos, index);
        Message::parse(&self.input[pos..index]).into()
    }
}

impl<'a> std::iter::FusedIterator for ParseIter<'a> {}
