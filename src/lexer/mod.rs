//! The lexer breaks down text information into tokens, which can be used to assemble syntax.

/// Represents all possible tokens.
#[derive(Debug, PartialEq)]
pub enum Token {
    // variables
    Equal,
    Identifier(String),
    Let,

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
            tokens.extend(self.lex_ln(line.split_whitespace()));
        }
        tokens
    }

    /// Lexes an individual line, split by spaces, and returns a vector of tokens.
    fn lex_ln<'a>(&mut self, words: impl Iterator<Item = &'a str>) -> Vec<Token> {
        let mut tokens = vec![];
        for word in words {
            use Token::*;
            tokens.push(match word {
                // assignments
                "let" => Let,
                "=" => Equal,

                // math
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

#[cfg(test)]
mod tests {
    use super::*;
    use Token::*;

    #[test]
    fn variable_assignment() {
        assert_eq!(
            Lexer::new().lex("let x".into()),
            vec![Let, Identifier("x".into()), Endl]
        );
    }

    #[test]
    fn math() {
        assert_eq!(
            Lexer::new().lex("1 + 1".into()),
            vec![Number(1.), Add, Number(1.), Endl]
        );
        assert_eq!(
            Lexer::new().lex("1 - 1".into()),
            vec![Number(1.), Sub, Number(1.), Endl]
        );
        assert_eq!(
            Lexer::new().lex("1 * 1".into()),
            vec![Number(1.), Mul, Number(1.), Endl]
        );
        assert_eq!(
            Lexer::new().lex("1 / 1".into()),
            vec![Number(1.), Div, Number(1.), Endl]
        );
    }
}
