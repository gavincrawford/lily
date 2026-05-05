//! Debugging implementations for `Token` types.
//! These are used as a nicer display visual for tokens so that they don't take up as much space if
//! a crash or assertion propogates one to `stderr`.

use crate::lexer::*;
use std::fmt::Debug;

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Identifier(id) => write!(f, "Identifier({})", resolve!(*id)),
            Token::Equal => write!(f, "Equal"),
            Token::Function => write!(f, "Function"),
            Token::Struct => write!(f, "Struct"),
            Token::Return => write!(f, "Return"),
            Token::Let => write!(f, "Let"),
            Token::Number(n) => write!(f, "Number({n:?})"),
            Token::Bool(b) => write!(f, "Bool({b:?})"),
            Token::Str(s) => write!(f, "Str({s:?})"),
            Token::Char(c) => write!(f, "Char({c:?})"),
            Token::Undefined => write!(f, "Undefined"),
            Token::If => write!(f, "If"),
            Token::Else => write!(f, "Else"),
            Token::While => write!(f, "While"),
            Token::BlockStart => write!(f, "BlockStart"),
            Token::BlockEnd => write!(f, "BlockEnd"),
            Token::Break => write!(f, "Break"),
            Token::ParenOpen => write!(f, "ParenOpen"),
            Token::ParenClose => write!(f, "ParenClose"),
            Token::BracketOpen => write!(f, "BracketOpen"),
            Token::BracketClose => write!(f, "BracketClose"),
            Token::LogicalNot => write!(f, "LogicalNot"),
            Token::LogicalEq => write!(f, "LogicalEq"),
            Token::LogicalNeq => write!(f, "LogicalNeq"),
            Token::LogicalG => write!(f, "LogicalG"),
            Token::LogicalGe => write!(f, "LogicalGe"),
            Token::LogicalL => write!(f, "LogicalL"),
            Token::LogicalLe => write!(f, "LogicalLe"),
            Token::LogicalAnd => write!(f, "LogicalAnd"),
            Token::LogicalOr => write!(f, "LogicalOr"),
            Token::Add => write!(f, "Add"),
            Token::Sub => write!(f, "Sub"),
            Token::Mul => write!(f, "Mul"),
            Token::Div => write!(f, "Div"),
            Token::Pow => write!(f, "Pow"),
            Token::Increment => write!(f, "Increment"),
            Token::Decrement => write!(f, "Decrement"),
            Token::Floor => write!(f, "Floor"),
            Token::Import => write!(f, "Import"),
            Token::As => write!(f, "As"),
            Token::Comma => write!(f, "Comma"),
            Token::Dot => write!(f, "Dot"),
            Token::New => write!(f, "New"),
            Token::Endl => write!(f, "Endl"),
        }
    }
}

impl Debug for TaggedToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} (line {}, [{}..{}])",
            self.kind(),
            self.line(),
            self.start(),
            self.end()
        )
    }
}
