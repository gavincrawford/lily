//! The lexer breaks down text information into tokens, which can be used to assemble syntax.

use std::str::SplitWhitespace;

/// Represents all possible tokens.
pub enum Token {
    Identifier,
    Let,
}

pub struct Lexer;
impl Lexer {
    /// Creates a new lexer.
    pub fn new() -> Self {
        Self
    }

    /// Lexes the provided file, as a string, into a vector of tokens.
    pub fn lex(&mut self, buf: String) -> Vec<Token> {
        // convert buf to lines by splitting on semicolons
        let buf = buf.split(";");

        // iterate over lines
        let mut tokens = vec![];
        for line in buf {
            tokens.extend(self.lex_ln(line.split_whitespace()));
        }

        // return tokens
        tokens
    }

    /// Lexes an individual line, split by spaces, and returns a vector of tokens.
    fn lex_ln(&mut self, words: SplitWhitespace) -> Vec<Token> {
        let mut tokens = vec![];
        for word in words {
            use Token::*;
            tokens.push(match word {
                _ => Identifier,
            });
        }
        tokens
    }
}
