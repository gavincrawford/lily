mod debug;
mod display;
mod from;

use crate::interner::Symbol;

/// Pairs a token with the source location it was lexed from.
///
/// Carries the line plus byte offsets `[start, end)` into the source buffer. Byte offsets enable
/// caret-style error rendering and source slicing (e.g. recovering the original lexeme of a
/// normalized token). The lexer produces a `Vec<SpannedToken>` via `Lexer::lex_spanned`; the parser
/// consumes these directly and strips the position info at the AST construction boundary so
/// runtime values never carry parse-time position data.
#[derive(PartialEq, Clone)]
pub struct SpannedToken {
    kind: Token,
    line: usize,
    start: usize,
    end: usize,
}

impl SpannedToken {
    /// Returns the source line on which this token was lexed.
    pub(crate) fn line(&self) -> usize {
        self.line
    }

    /// Returns the byte offset of the first byte of this token in the source buffer.
    pub(crate) fn start(&self) -> usize {
        self.start
    }

    /// Returns the byte offset one past the last byte of this token (exclusive).
    pub(crate) fn end(&self) -> usize {
        self.end
    }

    /// Returns a reference to the inner token kind, discarding the position info.
    pub(crate) fn kind(&self) -> &Token {
        &self.kind
    }
}

impl From<SpannedToken> for Token {
    fn from(value: SpannedToken) -> Self {
        value.kind
    }
}

impl PartialEq<SpannedToken> for Token {
    fn eq(&self, other: &SpannedToken) -> bool {
        *self == other.kind
    }
}

/// Represents all possible tokens.
#[derive(PartialEq, Clone)]
pub enum Token {
    // variables
    Equal,
    Identifier(Symbol),
    Function,
    Struct,
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
    While,
    BlockStart,
    BlockEnd,
    Break,
    Return,

    // delimiters
    ParenOpen,
    ParenClose,
    BracketOpen,
    BracketClose,

    // logic
    LogicalNot,
    LogicalEq,
    LogicalNeq,
    LogicalG,
    LogicalGe,
    LogicalL,
    LogicalLe,
    LogicalAnd,
    LogicalOr,

    // math ops
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Floor,
    Increment,
    Decrement,

    // modules
    Import,
    As,

    // other
    Comma,
    Dot,
    New,
    Endl,
}

impl Token {
    /// Returns true if `self` is an operator.
    /// Returns true for both numeric and logical operators.
    pub(crate) fn is_operator(&self) -> bool {
        matches!(
            self,
            Token::Add
                | Token::Sub
                | Token::Mul
                | Token::Div
                | Token::Floor
                | Token::Pow
                | Token::LogicalL
                | Token::LogicalLe
                | Token::LogicalG
                | Token::LogicalGe
                | Token::LogicalEq
                | Token::LogicalNeq
                | Token::LogicalAnd
                | Token::LogicalOr
        )
    }

    /// Returns true if `self` is a literal.
    /// Numbers, strings, chars, and booleans are all literal.
    pub(crate) fn is_literal(&self) -> bool {
        matches!(
            self,
            Token::Number(_) | Token::Str(_) | Token::Char(_) | Token::Bool(_)
        )
    }

    /// Returns a `SpannedToken` with the given line and byte-offset span attached.
    /// `end` is exclusive: a span of `[3, 5)` covers bytes 3 and 4.
    pub(crate) fn at(self, line: usize, start: usize, end: usize) -> SpannedToken {
        SpannedToken {
            kind: self,
            line,
            start,
            end,
        }
    }
}
