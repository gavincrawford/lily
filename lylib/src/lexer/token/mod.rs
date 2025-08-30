mod debug;
mod display;
mod from;

/// Represents all possible tokens.
#[derive(PartialEq, Clone)]
pub enum Token {
    // variables
    Equal,
    Identifier(usize),
    Function,
    Struct,
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
    While,
    BlockStart,
    BlockEnd,

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
        match self {
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
            | Token::LogicalOr => true,
            _ => false,
        }
    }

    /// Returns true if `self` is a literal.
    /// Numbers, strings, chars, and booleans are all literal.
    pub(crate) fn is_literal(&self) -> bool {
        matches!(self, Token::Number(_) | Token::Str(_) | Token::Char(_) | Token::Bool(_))
    }
}
