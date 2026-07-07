//! The lexer breaks down text information into tokens, which can be used to assemble syntax.

mod tests;
mod token;
mod util;

use anyhow::{Context, Result, bail};
use std::mem::take;

pub use token::{SpannedToken, Token};

/// Lexer capture mode.
enum CaptureMode {
    General,
    Number,
    Operator,
    String,
    Char,
    Comment,
}

/// The lexer transforms source code text into a sequence of tokens.
///
/// The lexer operates in different capture modes to handle various language constructs
/// such as numbers, strings, comments, and operators.
pub struct Lexer {
    /// State-machine mode.
    mode: CaptureMode,

    // Registers
    number_register: String,
    keyword_register: String,
    string_register: String,
    operator_register: Option<Token>,

    // Output buffer; drained into the caller via `mem::take`.
    tokens: Vec<SpannedToken>,
    /// Input buffer; contains entire source script.
    chars: Vec<char>,

    /// Line position counter.
    line: usize,
    /// Byte offset of the current char in the source buffer; the first char sits at 0
    char: usize,
    /// Start offset of an accumulating multi-char token (number/keyword/string/char)
    token_start: usize,
    /// Start offset of an in-progress operator-mode token (`=`, `<`, `>`, `!`, `&`, `|`, `+`, `-`,
    /// `*`, `/`, `^`)
    operator_start: usize,
}

impl Default for Lexer {
    fn default() -> Self {
        Self::new()
    }
}

impl Lexer {
    /// Creates a new lexer.
    pub fn new() -> Self {
        Self {
            mode: CaptureMode::General,
            number_register: String::new(),
            keyword_register: String::new(),
            string_register: String::new(),
            operator_register: None,
            tokens: vec![],
            chars: vec![],
            line: 1,
            char: 0,
            token_start: 0,
            operator_start: 0,
        }
    }

    /// Lexes the provided file, as a string, into a vector of tokens.
    pub fn lex(&mut self, buf: String) -> Result<Vec<Token>> {
        let res = self.lex_spanned(buf);
        res.map(|spanned| spanned.into_iter().map(Token::from).collect())
    }

    /// Lexes the provided file into spanned tokens, preserving line and byte-span information.
    pub fn lex_spanned(&mut self, buf: String) -> Result<Vec<SpannedToken>> {
        self.lex_internal(buf)
            .context(format!("on line {}", self.line))?;
        Ok(take(&mut self.tokens))
    }

    /// Internal lexer loop. Serves as main functional component of both public-facing lex
    /// functions.
    fn lex_internal(&mut self, buf: String) -> Result<()> {
        use Token::*;
        self.chars = buf.chars().collect();

        loop {
            let c = match self.chars.get(self.char) {
                Some(&c) => c,
                None => return Ok(()),
            };

            match self.mode {
                CaptureMode::General => {
                    match c {
                        // numbers
                        c if c.is_numeric() && self.keyword_register.is_empty() => {
                            self.mode = CaptureMode::Number;
                            self.token_start = self.char;
                            self.number_register.push(c);
                        }

                        // quotes, for str & char
                        '\"' => {
                            self.token_start = self.char;
                            self.mode = CaptureMode::String;
                        }
                        '\'' => {
                            self.token_start = self.char;
                            self.mode = CaptureMode::Char;
                        }

                        // keywords and identifiers
                        '(' | ')' | '[' | ']' | ',' | ' ' => {
                            if let Some(token) = Token::from_keyword(&self.keyword_register) {
                                // if the register contains a keyword, that takes priority
                                self.push_token(token, self.char);
                            } else if !self.keyword_register.is_empty() {
                                // otherwise, it'd be an identifier
                                self.push_token(
                                    Identifier(intern!(self.keyword_register.clone())),
                                    self.char,
                                );
                            }
                            self.keyword_register.clear();

                            // match delimiters
                            if let Some(token) = Token::from_char(c) {
                                self.tokens
                                    .push(token.at(self.line, self.char, self.char + 1));
                            }
                        }

                        // operators (=, <, >, !, &, |, +, -, *, /, ^); punctuation is matched
                        // above and never reaches this guard
                        c if Token::from_char(c).is_some() => {
                            // flush any pending identifier/keyword so it is emitted before this op
                            self.flush_keyword();
                            self.operator_register = Token::from_char(c);
                            self.operator_start = self.char;
                            self.mode = CaptureMode::Operator;
                        }

                        c if c.is_alphanumeric() || c == '_' => {
                            if self.keyword_register.is_empty() {
                                self.token_start = self.char;
                            }
                            self.keyword_register.push(c);
                        }
                        '.' => {
                            if !self.keyword_register.is_empty() {
                                self.push_token(
                                    Identifier(intern!(self.keyword_register.clone())),
                                    self.char,
                                );
                            }
                            self.keyword_register.clear();
                            self.tokens
                                .push(Dot.at(self.line, self.char, self.char + 1));
                        }

                        // endlines
                        ';' | '\n' => {
                            if let Some(token) = Token::from_keyword(&self.keyword_register) {
                                self.push_token(token, self.char);
                            } else if !self.keyword_register.is_empty() {
                                self.push_token(
                                    Identifier(intern!(self.keyword_register.clone())),
                                    self.char,
                                );
                            }
                            self.keyword_register.clear();
                            self.tokens
                                .push(Endl.at(self.line, self.char, self.char + 1));

                            // advance line counter only if this is an endline
                            // semicolons do not count as lines
                            if c == '\n' {
                                self.line += 1;
                            }
                        }

                        // comments
                        '#' => {
                            self.mode = CaptureMode::Comment;
                        }

                        // other
                        _ => {}
                    }
                }
                CaptureMode::Comment => {
                    if c == '\n' || c == ';' {
                        self.tokens
                            .push(Endl.at(self.line, self.char, self.char + 1));
                        self.mode = CaptureMode::General;
                    }
                }
                CaptureMode::Operator => {
                    if let Some(token) = &self.operator_register {
                        // 2-char operators span [operator_start, self.char + 1);
                        // 1-char fallbacks span [operator_start, operator_start + 1).
                        let two = self.char + 1;
                        let one = self.operator_start + 1;
                        match (token, c) {
                            (Equal, '=') => self.push_operator(LogicalEq, two),
                            (LogicalL, '=') => self.push_operator(LogicalLe, two),
                            (LogicalG, '=') => self.push_operator(LogicalGe, two),
                            (LogicalAnd, '&') => self.push_operator(LogicalAnd, two),
                            (LogicalOr, '|') => self.push_operator(LogicalOr, two),
                            (LogicalNot, '=') => self.push_operator(LogicalNeq, two),
                            (Add, '+') => self.push_operator(Increment, two),
                            (Sub, '-') => self.push_operator(Decrement, two),
                            (Div, '/') => self.push_operator(Floor, two),

                            // single-char fallbacks, these operators are valid standalone
                            (Equal, _)
                            | (LogicalL, _)
                            | (LogicalG, _)
                            | (LogicalNot, _)
                            | (Add, _)
                            | (Sub, _)
                            | (Mul, _)
                            | (Div, _)
                            | (Pow, _) => {
                                self.push_operator(token.clone(), one);
                                self.operator_register = None;
                                self.mode = CaptureMode::General;
                                // current char belongs to the next token, so `continue` skips back to
                                // `General` mode without skipping it
                                continue;
                            }
                            _ => unreachable!(),
                        }
                    }
                    self.operator_register = None;
                    self.mode = CaptureMode::General;
                }
                CaptureMode::Number => match c {
                    n if n.is_numeric() || n == '.' => {
                        self.number_register.push(n);
                    }
                    _ => {
                        if let Ok(number) = self.number_register.parse::<f32>() {
                            // number parsed ok-- push token. `self.char` is the offset of the
                            // delimiter that ended the number, which is one-past-end of
                            // the literal.
                            self.push_token(Number(number), self.char);
                            self.number_register.clear();
                        } else {
                            // number failed to parse, bail
                            bail!("cannot coerce {} to number", self.number_register);
                        }
                        self.mode = CaptureMode::General;
                        continue;
                    }
                },
                CaptureMode::String => match c {
                    '\"' => {
                        // closing quote consumed; end is one past it
                        self.push_token(Str(self.string_register.clone()), self.char + 1);
                        self.string_register.clear();
                        self.mode = CaptureMode::General;
                    }
                    _ => {
                        self.string_register.push(c);
                    }
                },
                CaptureMode::Char => {
                    match self.peek() {
                        Some(&'\'') => {
                            // push char token: token_start covers the opening quote, and the
                            // span ends one past the closing quote
                            self.push_token(Char(c), self.char + 2);

                            // skip closing quote; the loop advancement will move past it
                            self.char += 1;
                            self.mode = CaptureMode::General;
                        }
                        Some(_) => bail!("literals can only be one character long"),
                        None => bail!("expected char, found EOF"),
                    }
                }
            }

            self.char += 1;
        }
    }
}
