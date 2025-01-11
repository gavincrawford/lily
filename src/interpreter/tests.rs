#![cfg(test)]

use super::*;
use crate::{
    lexer::{Lexer, Token},
    parser::Parser,
};

#[test]
fn global_scope() {
    let mut i = Interpreter::new();
    i.execute(
        Parser::new(Lexer::new().lex("let a = 1; let b = \"string\"; let c = a;".into())).parse(),
    );
    assert_eq!(i.get("a".into()), Token::Number(1.));
    assert_eq!(i.get("b".into()), Token::Str("string".into()));
    assert_eq!(i.get("c".into()), i.get("a".into()));
}

#[test]
fn math() {
    let mut i = Interpreter::new();
    i.execute(Parser::new(Lexer::new().lex("let a = 1 + 2 * 2; let b = a + 1;".into())).parse());
    assert_eq!(i.get("a".into()), Token::Number(5.));
    assert_eq!(i.get("b".into()), Token::Number(6.));
}
