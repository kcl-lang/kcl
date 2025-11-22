use std::ops::Deref;

use crate::token::Token;

/// A `TokenStream` is an abstract sequence of tokens.
#[derive(Clone, Debug, Default)]
pub struct TokenStream(pub(crate) Vec<Token>);

impl TokenStream {
    pub fn new(streams: Vec<Token>) -> TokenStream {
        TokenStream(streams)
    }

    pub fn cursor(self) -> Cursor {
        Cursor::new(self)
    }
}

impl Deref for TokenStream {
    type Target = Vec<Token>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct Cursor {
    /// Token stream
    pub stream: TokenStream,
    /// Cursor index
    index: usize,
}

impl Cursor {
    fn new(stream: TokenStream) -> Self {
        Cursor { stream, index: 0 }
    }

    /// Get the next token index.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Peek next token.
    pub fn peek(&self) -> Option<Token> {
        if self.index < self.stream.len() {
            Some(self.stream[self.index])
        } else {
            None
        }
    }

    /// Peek next two token.
    pub fn peek2(&self) -> Option<Token> {
        if self.index + 1 < self.stream.len() {
            Some(self.stream[self.index + 1])
        } else {
            None
        }
    }
}

impl Iterator for Cursor {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        if self.index < self.stream.len() {
            self.index += 1;
            Some(self.stream[self.index - 1])
        } else {
            None
        }
    }
}
