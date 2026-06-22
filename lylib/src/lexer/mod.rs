//! The lexer breaks down text information into tokens, which can be used to assemble syntax.

mod token;
pub use token::{SpannedToken, Token};

use anyhow::{Context, Result, bail};
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
    mode: CaptureMode,
    number_register: String,
    keyword_register: String,
    string_register: String,
    equality_register: Option<Token>,

    /// Line position counter.
    line_n: usize,
    /// Byte offset of the current `c` in the source buffer; the first char sits at 0
    char_n: usize,
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
            equality_register: None,
            line_n: 1,
            char_n: 0,
        }
    }

    /// Lexes the provided file, as a string, into a vector of tokens.
    pub fn lex(&mut self, buf: String) -> Result<Vec<Token>> {
        let res = self.lex_spanned(buf);
        res.map(|spanned| spanned.into_iter().map(Token::from).collect())
    }

    /// Lexes the provided file into spanned tokens, preserving line and byte-span information.
    pub fn lex_spanned(&mut self, buf: String) -> Result<Vec<SpannedToken>> {
        use Token::*;
        let mut chars = buf.chars().peekable();
        let mut tokens: Vec<SpannedToken> = vec![];
        let mut c = chars.next().context("source file empty")?;

        // start offset of an accumulating multi-char token (number/keyword/string/char)
        let mut token_start: usize = 0;
        // start offset of an in-progress equality-mode operator (`=`, `<`, `>`, `!`, `&`, `|`)
        let mut equality_start: usize = 0;

        let res = (|| {
            loop {
                match self.mode {
                    CaptureMode::General => {
                        match c {
                            // TODO this should just get moved out to its own mode vvv

                            // operators
                            '+' => self.long_op(
                                &mut chars,
                                &mut tokens,
                                token_start,
                                '+',
                                Increment,
                                Add,
                            ),
                            '-' => self.long_op(
                                &mut chars,
                                &mut tokens,
                                token_start,
                                '-',
                                Decrement,
                                Sub,
                            ),
                            '*' => tokens.push(Mul.at(
                                self.line_n,
                                self.char_n,
                                self.char_n + c.len_utf8(),
                            )),
                            '/' => {
                                self.long_op(&mut chars, &mut tokens, token_start, '/', Floor, Div)
                            }
                            '^' => tokens.push(Pow.at(
                                self.line_n,
                                self.char_n,
                                self.char_n + c.len_utf8(),
                            )),

                            // equalities
                            '=' => {
                                self.equality_register = Some(Equal);
                                equality_start = self.char_n;
                                self.mode = CaptureMode::Equality;
                            }
                            '!' => {
                                self.equality_register = Some(LogicalNot);
                                equality_start = self.char_n;
                                self.mode = CaptureMode::Equality;
                            }
                            '>' => {
                                self.equality_register = Some(LogicalG);
                                equality_start = self.char_n;
                                self.mode = CaptureMode::Equality;
                            }
                            '<' => {
                                self.equality_register = Some(LogicalL);
                                equality_start = self.char_n;
                                self.mode = CaptureMode::Equality;
                            }
                            '&' => {
                                self.equality_register = Some(LogicalAnd);
                                equality_start = self.char_n;
                                self.mode = CaptureMode::Equality;
                            }
                            '|' => {
                                self.equality_register = Some(LogicalOr);
                                equality_start = self.char_n;
                                self.mode = CaptureMode::Equality;
                            }

                            // numbers
                            c if c.is_numeric() && self.keyword_register.is_empty() => {
                                self.mode = CaptureMode::Number;
                                token_start = self.char_n;
                                self.number_register.push(c);
                            }

                            // quotes, for str & char
                            '\"' => {
                                token_start = self.char_n;
                                self.mode = CaptureMode::String;
                            }
                            '\'' => {
                                token_start = self.char_n;
                                self.mode = CaptureMode::Char;
                            }

                            // keywords and identifiers
                            '(' | ')' | '[' | ']' | ',' | ' ' => {
                                if let Some(token) = self.keyword_from_register() {
                                    // if the register contains a keyword, that takes priority
                                    tokens.push(token.at(self.line_n, token_start, self.char_n));
                                } else if !self.keyword_register.is_empty() {
                                    // otherwise, it'd be an identifier
                                    tokens.push(
                                        Identifier(intern!(self.keyword_register.clone())).at(
                                            self.line_n,
                                            token_start,
                                            self.char_n,
                                        ),
                                    );
                                }
                                self.keyword_register.clear();

                                // match delimiters
                                let end = self.char_n + c.len_utf8();
                                match c {
                                    '(' => tokens.push(ParenOpen.at(self.line_n, self.char_n, end)),
                                    ')' => {
                                        tokens.push(ParenClose.at(self.line_n, self.char_n, end))
                                    }
                                    '[' => {
                                        tokens.push(BracketOpen.at(self.line_n, self.char_n, end))
                                    }
                                    ']' => {
                                        tokens.push(BracketClose.at(self.line_n, self.char_n, end))
                                    }
                                    ',' => tokens.push(Comma.at(self.line_n, self.char_n, end)),
                                    _ => {}
                                }
                            }
                            c if c.is_alphanumeric() || c == '_' => {
                                if self.keyword_register.is_empty() {
                                    token_start = self.char_n;
                                }
                                self.keyword_register.push(c);
                            }
                            '.' => {
                                if !self.keyword_register.is_empty() {
                                    tokens.push(
                                        Identifier(intern!(self.keyword_register.clone())).at(
                                            self.line_n,
                                            token_start,
                                            self.char_n,
                                        ),
                                    );
                                }
                                self.keyword_register.clear();
                                tokens.push(Dot.at(
                                    self.line_n,
                                    self.char_n,
                                    self.char_n + c.len_utf8(),
                                ));
                            }

                            // endlines
                            ';' | '\n' => {
                                if let Some(token) = self.keyword_from_register() {
                                    tokens.push(token.at(self.line_n, token_start, self.char_n));
                                } else if !self.keyword_register.is_empty() {
                                    tokens.push(
                                        Identifier(intern!(self.keyword_register.clone())).at(
                                            self.line_n,
                                            token_start,
                                            self.char_n,
                                        ),
                                    );
                                }
                                self.keyword_register.clear();
                                tokens.push(Endl.at(
                                    self.line_n,
                                    self.char_n,
                                    self.char_n + c.len_utf8(),
                                ));

                                // advance line counter only if this is an endline
                                // semicolons do not count as lines
                                if c == '\n' {
                                    self.line_n += 1;
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
                            tokens.push(Endl.at(
                                self.line_n,
                                self.char_n,
                                self.char_n + c.len_utf8(),
                            ));
                            self.mode = CaptureMode::General;
                        }
                    }
                    CaptureMode::Equality => {
                        if let Some(token) = &self.equality_register {
                            // 2-char operators span [equality_start..pos + c.len_utf8());
                            // 1-char fallbacks span [equality_start..equality_start + 1).
                            let two = self.char_n + c.len_utf8();
                            let one = equality_start + 1;
                            match (token, c) {
                                (Equal, '=') => {
                                    tokens.push(LogicalEq.at(self.line_n, equality_start, two))
                                }
                                (Equal, _) => {
                                    tokens.push(Equal.at(self.line_n, equality_start, one))
                                }
                                (LogicalL, '=') => {
                                    tokens.push(LogicalLe.at(self.line_n, equality_start, two))
                                }
                                (LogicalL, _) => {
                                    tokens.push(LogicalL.at(self.line_n, equality_start, one))
                                }
                                (LogicalG, '=') => {
                                    tokens.push(LogicalGe.at(self.line_n, equality_start, two))
                                }
                                (LogicalG, _) => {
                                    tokens.push(LogicalG.at(self.line_n, equality_start, one))
                                }
                                (LogicalAnd, '&') => {
                                    tokens.push(LogicalAnd.at(self.line_n, equality_start, two))
                                }
                                (LogicalOr, '|') => {
                                    tokens.push(LogicalOr.at(self.line_n, equality_start, two))
                                }
                                (LogicalNot, '=') => {
                                    tokens.push(LogicalNeq.at(self.line_n, equality_start, two))
                                }
                                (LogicalNot, _) => {
                                    // NOTE:
                                    // this bit is required to skip the character advancement that
                                    // occurs for all of the other branches here. this specifically
                                    // fixes double negatives (`!!true`). it's likely that there's
                                    // other bugs similar to this one that might need this workaround
                                    tokens.push(LogicalNot.at(self.line_n, equality_start, one));
                                    self.equality_register = None;
                                    self.mode = CaptureMode::General;
                                    continue;
                                }
                                _ => {
                                    unreachable!()
                                }
                            }
                        }
                        self.equality_register = None;
                        self.mode = CaptureMode::General;
                    }
                    CaptureMode::Number => match c {
                        n if n.is_numeric() || n == '.' => {
                            self.number_register.push(n);
                        }
                        _ => {
                            if let Ok(number) = self.number_register.parse::<f32>() {
                                // number parsed ok-- push token. `pos` is the offset of the
                                // delimiter that ended the number, which is one-past-end of
                                // the literal.
                                tokens.push(Number(number).at(
                                    self.line_n,
                                    token_start,
                                    self.char_n,
                                ));
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
                            tokens.push(Str(self.string_register.clone()).at(
                                self.line_n,
                                token_start,
                                self.char_n + c.len_utf8(),
                            ));
                            self.string_register.clear();
                            self.mode = CaptureMode::General;
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

                            // skip second quote (manually advance pos to keep it in sync)
                            chars.next();
                            let close_len = '\''.len_utf8();

                            // push char token: token_start covers the opening quote, and the
                            // span ends one past the closing quote we just consumed
                            tokens.push(Char(c).at(
                                self.line_n,
                                token_start,
                                self.char_n + c.len_utf8() + close_len,
                            ));
                            self.char_n += close_len;
                            self.mode = CaptureMode::General;
                        } else {
                            // if no char is found, this is an EOF
                            bail!("expected char, found EOF");
                        }
                    }
                }
                if let Some(next_c) = chars.next() {
                    self.char_n += c.len_utf8();
                    c = next_c;
                } else {
                    return Ok(tokens);
                }
            }
        })();

        res.context(format!("on line {}", self.line_n))
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

    /// Handles long operators, such as `++`, `--`, & `//`.
    fn long_op(
        &mut self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        tokens: &mut Vec<SpannedToken>,
        token_start: usize,
        expected_char: char,
        double_token: Token,
        single_token: Token,
    ) {
        // flush any pending identifier/keyword so it is emitted *before* this operator
        // (e.x., `a++` lexes as `[Identifier(a), Increment]`, not the reverse)
        self.flush_keyword(tokens, token_start);

        let start = self.char_n;
        if let Some(peek_char) = chars.peek() {
            if *peek_char == expected_char {
                chars.next();
                // both the leading and trailing chars here are ASCII, so 1+1 byte span
                tokens.push(double_token.at(self.line_n, start, start + 2));
                self.char_n += 1;
            } else {
                tokens.push(single_token.at(self.line_n, start, start + 1));
            }
        } else {
            tokens.push(single_token.at(self.line_n, start, start + 1));
        }
    }

    /// Handles double-character operators like `++`, `--`, `//`. Used to forcefully emit keywords
    /// for use cases such as postfix unaries.
    ///
    /// Flushes the pending keyword register into `tokens` as a keyword (if it matches one) or an
    /// identifier, then clears the register. A no-op if the register is empty.
    fn flush_keyword(&mut self, tokens: &mut Vec<SpannedToken>, token_start: usize) {
        if let Some(token) = self.keyword_from_register() {
            tokens.push(token.at(self.line_n, token_start, self.char_n));
        } else if !self.keyword_register.is_empty() {
            tokens.push(
                Token::Identifier(intern!(self.keyword_register.clone())).at(
                    self.line_n,
                    token_start,
                    self.char_n,
                ),
            );
        }
        self.keyword_register.clear();
    }
}
