#![cfg(test)]

use super::*;
use crate::{
    lexer::{Lexer, Token},
    parser::Parser,
};

#[test]
fn global_variables() {
    let mut i = Interpreter::new();
    let ast = Parser::new(Lexer::new().lex(include_str!("tests/global_variables.ly").to_string()))
        .parse();
    i.execute(&ast);
    assert_eq!(i.get("a".into()), Token::Number(1.));
    assert_eq!(i.get("b".into()), Token::Bool(true));
    assert_eq!(i.get("c".into()), Token::Str("str".into()));
    assert_eq!(i.get("d".into()), Token::Char('c'));
}

#[test]
fn math() {
    let mut i = Interpreter::new();
    let ast = Parser::new(Lexer::new().lex(include_str!("tests/math.ly").to_string())).parse();
    i.execute(&ast);
    assert_eq!(i.get("a".into()), Token::Number(2.));
    assert_eq!(i.get("b".into()), Token::Number(0.));
    assert_eq!(i.get("c".into()), Token::Number(25.));
    assert_eq!(i.get("d".into()), Token::Number(2.));
    assert_eq!(i.get("e".into()), Token::Number(6.));
}

#[test]
fn conditionals() {
    let mut i = Interpreter::new();
    let ast =
        Parser::new(Lexer::new().lex(include_str!("tests/conditionals.ly").to_string())).parse();
    i.execute(&ast);
    assert_eq!(i.get("a".into()), Token::Number(5.));
}

#[test]
fn functions() {
    let mut i = Interpreter::new();
    let ast = Parser::new(Lexer::new().lex(include_str!("tests/functions.ly").to_string())).parse();
    i.execute(&ast);
    assert_eq!(i.get("a".into()), Token::Number(10.));
    assert_eq!(i.get("b".into()), Token::Number(20.));
    assert_eq!(i.get("c".into()), Token::Bool(true));
}

#[test]
fn loops() {
    let mut i = Interpreter::new();
    let ast = Parser::new(Lexer::new().lex(include_str!("tests/loops.ly").to_string())).parse();
    i.execute(&ast);
    assert_eq!(i.get("i".into()), Token::Number(25.));
    assert_eq!(i.get("a".into()), Token::Number(25.));
}
