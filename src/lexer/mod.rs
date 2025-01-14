//! The lexer breaks down text information into tokens, which can be used to assemble syntax.

mod tests;

/// Represents all possible tokens.
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // variables
    Equal,
    Identifier(String),
    Function,
    Return,
    Let,

    // data types
    Number(f32),
    Bool(bool),
    Str(String),
    Char(char),
    Undefined,

    // conditionals
    If,
    Else,
    BlockStart,
    BlockEnd,
    ParenOpen,
    ParenClose,

    // logic
    LogicalNot,
    LogicalEq,
    LogicalNeq,
    LogicalG,
    LogicalGe,
    LogicalL,
    LogicalLe,

    // math ops
    Add,
    Sub,
    Mul,
    Div,

    // other
    Comma,
    Endl,
}

/// Lexer capture mode.
enum CaptureMode {
    General,
    Number,
    Equality,
    String,
    Char,
}

pub struct Lexer {
    number_register: String,
    keyword_register: String,
    string_register: String,
    equality_register: Option<Token>,
}
impl Lexer {
    /// Creates a new lexer.
    pub fn new() -> Self {
        Self {
            number_register: String::new(),
            keyword_register: String::new(),
            string_register: String::new(),
            equality_register: None,
        }
    }

    /// Lexes the provided file, as a string, into a vector of tokens.
    pub fn lex(&mut self, buf: String) -> Vec<Token> {
        use Token::*;
        let buf = buf.replace("\n", ";");
        let mut chars = buf.chars().peekable();
        let mut tokens = vec![];
        let mut mode = CaptureMode::General;
        let mut c = chars.next().expect("source file empty.");
        loop {
            match mode {
                CaptureMode::General => {
                    match c {
                        // operators
                        '+' => {
                            tokens.push(Add);
                        }
                        '-' => {
                            tokens.push(Sub);
                        }
                        '*' => {
                            tokens.push(Mul);
                        }
                        '/' => {
                            tokens.push(Div);
                        }

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

                        // numbers
                        c if c.is_numeric() => {
                            mode = CaptureMode::Number;
                            self.number_register.push(c);
                        }

                        // non-number words
                        '\"' => {
                            mode = CaptureMode::String;
                        }
                        '\'' => {
                            mode = CaptureMode::Char;
                        }
                        '(' | ')' | ',' => {
                            // add identifier for function calls
                            if !self.keyword_register.is_empty() {
                                tokens.push(Identifier(self.keyword_register.clone()));
                                self.keyword_register.clear();
                            }
                            match c {
                                '(' => tokens.push(ParenOpen),
                                ')' => tokens.push(ParenClose),
                                ',' => tokens.push(Comma),
                                _ => panic!(),
                            }
                        }
                        c if c.is_alphanumeric() || c == '_' => {
                            self.keyword_register.push(c);
                        }

                        // keywords
                        ' ' => {
                            if let Some(token) = self.keyword_from_register() {
                                tokens.push(token);
                            } else if !self.keyword_register.is_empty() {
                                tokens.push(Identifier(self.keyword_register.clone()));
                            }
                            self.keyword_register.clear();
                        }

                        // endlines
                        ';' | '\n' => {
                            if let Some(token) = self.keyword_from_register() {
                                tokens.push(token);
                            } else if !self.keyword_register.is_empty() {
                                tokens.push(Identifier(self.keyword_register.clone()));
                            }
                            self.keyword_register.clear();
                            tokens.push(Endl);
                        }

                        // other
                        _ => {}
                    }
                }
                CaptureMode::Equality => {
                    if let Some(token) = &self.equality_register {
                        match (token, c) {
                            (Equal, '=') => tokens.push(LogicalEq),
                            (Equal, _) => tokens.push(Equal),
                            (LogicalNot, '=') => tokens.push(LogicalNeq),
                            (LogicalL, '=') => tokens.push(LogicalLe),
                            (LogicalL, _) => tokens.push(LogicalL),
                            (LogicalG, '=') => tokens.push(LogicalGe),
                            (LogicalG, _) => tokens.push(LogicalG),
                            _ => {}
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
                            // number failed to parse, panic
                            panic!("cannot coerce {} to number.", self.number_register);
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
                    // peek ahead to make sure the char is 1 in length
                    let next = chars.peek().expect("expected char, found EOF.");
                    if *next != '\'' {
                        panic!("literals can only be one character long.");
                    }

                    // skip second quote
                    chars.next();

                    // push char token
                    tokens.push(Char(c));
                    mode = CaptureMode::General;
                }
            }
            if let Some(next_c) = chars.next() {
                c = next_c;
            } else {
                return tokens;
            }
        }
    }

    /// Return the enum variant of the keyword stored in the keyword register.
    fn keyword_from_register(&self) -> Option<Token> {
        use Token::*;
        match &*self.keyword_register {
            "let" => Some(Let),
            "func" => Some(Function),
            "return" => Some(Return),
            "if" => Some(If),
            "else" => Some(Else),
            "do" => Some(BlockStart),
            "end" => Some(BlockEnd),
            "true" => Some(Bool(true)),
            "false" => Some(Bool(false)),
            _ => None,
        }
    }
}
