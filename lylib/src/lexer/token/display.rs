use super::*;
use std::fmt::Display;

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Token::*;
        match self {
            Bool(v) => write!(f, "{v}"),
            Str(v) => write!(f, "{v}"),
            Number(v) => write!(f, "{v}"),
            Char(v) => write!(f, "{v}"),
            Identifier(v) => write!(f, "{v}"),
            _ => write!(f, "{self:?}"),
        }?;
        Ok(())
    }
}
