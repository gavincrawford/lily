mod display;
mod from;

/// Represents all possible tokens.
#[derive(Debug, PartialEq, Clone)]
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
    New,
    Endl,
}
