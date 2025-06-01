//! This module provides the implementations to convert from basic types to their relevant tokens.

use super::Token;

impl From<isize> for Token {
    fn from(value: isize) -> Self {
        Token::Number(value as f32)
    }
}

impl From<bool> for Token {
    fn from(value: bool) -> Self {
        Token::Bool(value)
    }
}

impl From<char> for Token {
    fn from(value: char) -> Self {
        Token::Char(value)
    }
}

impl From<&str> for Token {
    fn from(value: &str) -> Self {
        Token::Str(String::from(value))
    }
}
