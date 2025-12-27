//! The lexer breaks down text information into tokens, which can be used to assemble syntax.

mod token;
pub use token::Token;

use anyhow::{bail, Context, Result};
mod tests;

/// Lexer capture mode.
enum CaptureMode {
    General,
    Number,
    Equality,
    String,
    Char,
    Comment,
}

/// The lexer transforms source code text into a sequence of tokens.
///
/// The lexer operates in different capture modes to handle various language constructs
/// such as numbers, strings, comments, and operators.
pub struct Lexer {
    number_register: String,
    keyword_register: String,
    string_register: String,
    equality_register: Option<Token>,
}

impl Lexer {
    /// Creates a new lexer with the provided interner.
    pub fn new() -> Self {
        Self {
            number_register: String::new(),
            keyword_register: String::new(),
            string_register: String::new(),
            equality_register: None,
        }
    }

    /// Lexes the provided file, as a string, into a vector of tokens.
    pub fn lex(&mut self, buf: String) -> Result<Vec<Token>> {
        use Token::*;
        let buf = buf.replace("\n", ";");
        let mut chars = buf.chars().peekable();
        let mut tokens = vec![];
        let mut mode = CaptureMode::General;
        let mut c = chars.next().context("source file empty")?;
        loop {
            match mode {
                CaptureMode::General => {
                    match c {
                        // TODO this should just get moved out to its own mode vvv

                        // operators
                        '+' => self.long_op(&mut chars, &mut tokens, '+', Increment, Add),
                        '-' => self.long_op(&mut chars, &mut tokens, '-', Decrement, Sub),
                        '*' => tokens.push(Mul),
                        '/' => self.long_op(&mut chars, &mut tokens, '/', Floor, Div),
                        '^' => tokens.push(Pow),

                        // equalities
                        '=' => {
                            self.equality_register = Some(Equal);
                            mode = CaptureMode::Equality;
                        }
                        '!' => {
                            self.equality_register = Some(LogicalNot);
                            mode = CaptureMode::Equality;
                        }
                        '>' => {
                            self.equality_register = Some(LogicalG);
                            mode = CaptureMode::Equality;
                        }
                        '<' => {
                            self.equality_register = Some(LogicalL);
                            mode = CaptureMode::Equality;
                        }
                        '&' => {
                            self.equality_register = Some(LogicalAnd);
                            mode = CaptureMode::Equality;
                        }
                        '|' => {
                            self.equality_register = Some(LogicalOr);
                            mode = CaptureMode::Equality;
                        }

                        // numbers
                        c if c.is_numeric() && self.keyword_register.is_empty() => {
                            mode = CaptureMode::Number;
                            self.number_register.push(c);
                        }

                        // quotes, for str & char
                        '\"' => {
                            mode = CaptureMode::String;
                        }
                        '\'' => {
                            mode = CaptureMode::Char;
                        }

                        // keywords and identifiers
                        '(' | ')' | '[' | ']' | ',' | ' ' => {
                            if let Some(token) = self.keyword_from_register() {
                                // if the register contains a keyword, that takes priority
                                tokens.push(token);
                            } else if !self.keyword_register.is_empty() {
                                // otherwise, it'd be an identifier
                                tokens.push(Identifier(intern!(self.keyword_register.clone())));
                            }
                            self.keyword_register.clear();

                            // match delimiters
                            match c {
                                '(' => tokens.push(ParenOpen),
                                ')' => tokens.push(ParenClose),
                                '[' => tokens.push(BracketOpen),
                                ']' => tokens.push(BracketClose),
                                ',' => tokens.push(Comma),
                                _ => {}
                            }
                        }
                        c if c.is_alphanumeric() || c == '_' => {
                            self.keyword_register.push(c);
                        }
                        '.' => {
                            if !self.keyword_register.is_empty() {
                                tokens.push(Identifier(intern!(self.keyword_register.clone())));
                            }
                            self.keyword_register.clear();
                            tokens.push(Dot);
                        }

                        // endlines
                        ';' | '\n' => {
                            if let Some(token) = self.keyword_from_register() {
                                tokens.push(token);
                            } else if !self.keyword_register.is_empty() {
                                tokens.push(Identifier(intern!(self.keyword_register.clone())));
                            }
                            self.keyword_register.clear();
                            tokens.push(Endl);
                        }

                        // comments
                        '#' => {
                            mode = CaptureMode::Comment;
                        }

                        // other
                        _ => {}
                    }
                }
                CaptureMode::Comment => {
                    if c == '\n' || c == ';' {
                        tokens.push(Endl);
                        mode = CaptureMode::General;
                    }
                }
                CaptureMode::Equality => {
                    if let Some(token) = &self.equality_register {
                        match (token, c) {
                            (Equal, '=') => tokens.push(LogicalEq),
                            (Equal, _) => tokens.push(Equal),
                            (LogicalL, '=') => tokens.push(LogicalLe),
                            (LogicalL, _) => tokens.push(LogicalL),
                            (LogicalG, '=') => tokens.push(LogicalGe),
                            (LogicalG, _) => tokens.push(LogicalG),
                            (LogicalAnd, '&') => tokens.push(LogicalAnd),
                            (LogicalOr, '|') => tokens.push(LogicalOr),
                            (LogicalNot, '=') => tokens.push(LogicalNeq),
                            (LogicalNot, _) => {
                                // NOTE:
                                // this bit is required to skip the character advancement that
                                // occurs for all of the other branches here. this specifically
                                // fixes double negatives (`!!true`). it's likely that there's
                                // other bugs similar to this one that might need this workaround
                                tokens.push(LogicalNot);
                                self.equality_register = None;
                                mode = CaptureMode::General;
                                continue;
                            }
                            _ => {
                                unreachable!()
                            }
                        }
                    }
                    self.equality_register = None;
                    mode = CaptureMode::General;
                }
                CaptureMode::Number => match c {
                    n if n.is_numeric() || n == '.' => {
                        self.number_register.push(n);
                    }
                    _ => {
                        if let Ok(number) = self.number_register.parse::<f32>() {
                            // number parsed ok-- push token
                            tokens.push(Number(number));
                            self.number_register.clear();
                        } else {
                            // number failed to parse, bail
                            bail!("cannot coerce {} to number", self.number_register);
                        }
                        mode = CaptureMode::General;
                        continue;
                    }
                },
                CaptureMode::String => match c {
                    '\"' => {
                        tokens.push(Str(self.string_register.clone()));
                        self.string_register.clear();
                        mode = CaptureMode::General;
                    }
                    _ => {
                        self.string_register.push(c);
                    }
                },
                CaptureMode::Char => {
                    if let Some(next) = chars.peek() {
                        // peek ahead to make sure the char is 1 in length
                        if *next != '\'' {
                            bail!("literals can only be one character long");
                        }

                        // skip second quote
                        chars.next();

                        // push char token
                        tokens.push(Char(c));
                        mode = CaptureMode::General;
                    } else {
                        // if no char is found, this is an EOF
                        bail!("expected char, found EOF");
                    }
                }
            }
            if let Some(next_c) = chars.next() {
                c = next_c;
            } else {
                return Ok(tokens);
            }
        }
    }

    /// Return the enum variant of the keyword stored in the keyword register.
    fn keyword_from_register(&self) -> Option<Token> {
        use Token::*;
        match &*self.keyword_register {
            "let" => Some(Let),
            "new" => Some(New),
            "func" => Some(Function),
            "struct" => Some(Struct),
            "return" => Some(Return),
            "if" => Some(If),
            "else" => Some(Else),
            "while" => Some(While),
            "break" => Some(Break),
            "do" => Some(BlockStart),
            "end" => Some(BlockEnd),
            "true" => Some(Bool(true)),
            "false" => Some(Bool(false)),
            "import" => Some(Import),
            "as" => Some(As),
            _ => None,
        }
    }

    /// Handles double-character operators like ++, --, //.
    fn long_op(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        tokens: &mut Vec<Token>,
        expected_char: char,
        double_token: Token,
        single_token: Token,
    ) {
        if let Some(peek_char) = chars.peek() {
            if *peek_char == expected_char {
                chars.next();
                tokens.push(double_token);
            } else {
                tokens.push(single_token);
            }
        } else {
            tokens.push(single_token);
        }
    }
}
