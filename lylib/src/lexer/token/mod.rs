mod debug;
mod display;
mod from;

/// Pairs a token with the source line it was lexed from.
///
/// Used by the parser to attach line numbers to error messages. The lexer produces a
/// `Vec<TaggedToken>` via `Lexer::lex_tagged`; the parser consumes these directly and strips the
/// line tag at the AST construction boundary so runtime values never carry parse-time line data.
#[derive(PartialEq, Clone)]
pub struct TaggedToken {
    kind: Token,
    line: usize,
}

impl TaggedToken {
    /// Returns the source line on which this token was lexed.
    pub(crate) fn line(&self) -> usize {
        self.line
    }

    /// Returns a reference to the inner token kind, discarding the line tag.
    pub(crate) fn kind(&self) -> &Token {
        &self.kind
    }
}

impl From<TaggedToken> for Token {
    fn from(value: TaggedToken) -> Self {
        value.kind
    }
}

impl PartialEq<TaggedToken> for Token {
    fn eq(&self, other: &TaggedToken) -> bool {
        *self == other.kind
    }
}

/// Represents all possible tokens.
#[derive(PartialEq, Clone)]
pub enum Token {
    // variables
    Equal,
    Identifier(usize),
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

    /// Returns a tagged token with the given line attached.
    pub(crate) fn at_line(&self, line: usize) -> TaggedToken {
        TaggedToken {
            kind: self.clone(),
            line,
        }
    }
}
