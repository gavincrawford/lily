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
    i.execute(
        Parser::new(
            Lexer::new().lex("let a = 2 * 2 + 1; let b = a + 1; let c = (2 * 2) + 1;".into()),
        )
        .parse(),
    );
    assert_eq!(i.get("a".into()), Token::Number(6.));
    assert_eq!(i.get("b".into()), Token::Number(7.));
    assert_eq!(i.get("c".into()), Token::Number(5.));
}

#[test]
fn comparisons() {
    let mut i = Interpreter::new();
    i.execute(
        Parser::new(
            Lexer::new().lex("let a = 1; let b = 2; let a_g_b = a > b; let a_l_b = a < b;".into()),
        )
        .parse(),
    );
    assert_eq!(i.get("a_g_b".into()), Token::Bool(false));
    assert_eq!(i.get("a_l_b".into()), Token::Bool(true));
}

#[test]
fn conditionals() {
    let mut i = Interpreter::new();
    i.execute(Parser::new(Lexer::new().lex("let a = false;".into())).parse());
    assert_eq!(i.get("a".into()), Token::Bool(false));
    i.execute(Parser::new(Lexer::new().lex("if 2 > 1 do; a = true; end;".into())).parse());
    assert_eq!(i.get("a".into()), Token::Bool(true));
    i.execute(Parser::new(Lexer::new().lex("if 1 < 2 do; a = false; end;".into())).parse());
    assert_eq!(i.get("a".into()), Token::Bool(false));
    i.execute(Parser::new(Lexer::new().lex("if true do; a = true; end;".into())).parse());
    assert_eq!(i.get("a".into()), Token::Bool(true));
}

#[test]
fn scope() {
    let mut i = Interpreter::new();
    i.execute(
        Parser::new(
            Lexer::new().lex("let a = \"global\"; if true do; let b = \"local\"; end;".into()),
        )
        .parse(),
    );
    assert_eq!(i.get("a".into()), Token::Str("global".into()));
    assert_eq!(i.get("b".into()), Token::Undefined);
}
