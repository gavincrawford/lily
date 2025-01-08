//! The lexer breaks down text information into tokens, which can be used to assemble syntax.

mod tests;

/// Represents all possible tokens.
#[derive(Debug, PartialEq)]
pub enum Token {
    // variables
    Equal,
    Identifier(String),
    Let,

    // conditionals
    If,
    Else,
    BlockStart,
    BlockEnd,

    // logic
    LogicalEq,
    LogicalNeq,
    LogicalG,
    LogicalL,

    // math
    Number(f32),
    Add,
    Sub,
    Mul,
    Div,

    // other
    Endl,
}

pub struct Lexer;
impl Lexer {
    /// Creates a new lexer.
    pub fn new() -> Self {
        Self
    }

    /// Lexes the provided file, as a string, into a vector of tokens.
    pub fn lex(&mut self, buf: String) -> Vec<Token> {
        // remove unneeded escapes and split by ';'
        let buf = buf.replace(";", "\n");
        let buf = buf.split("\n");

        // generate tokens
        let mut tokens = vec![];
        for line in buf {
            // skip blank lines
            if line == "" {
                continue;
            }

            // parse line and store tokens
            tokens.extend(self.lex_ln(line));
        }
        tokens
    }

    /// Lexes an individual line and returns a vector of tokens.
    fn lex_ln(&mut self, words: &str) -> Vec<Token> {
        let mut tokens = vec![];
        for word in words.split_whitespace() {
            use Token::*;
            tokens.push(match word {
                // variables
                "let" => Let,
                "=" => Equal,

                // conditionals & logic
                "if" => If,
                "else" => Else,
                "then" => BlockStart,
                "end" => BlockEnd,
                "==" => LogicalEq,
                "!=" => LogicalNeq,
                ">" => LogicalG,
                "<" => LogicalL,

                // math & numbers
                s if s.parse::<f32>().is_ok() => Number(s.parse::<f32>().unwrap()),
                "+" => Add,
                "-" => Sub,
                "*" => Mul,
                "/" => Div,

                // other
                "--" => break,
                _ => Identifier(word.into()),
            });
        }
        tokens.push(Token::Endl);
        tokens
    }
}
