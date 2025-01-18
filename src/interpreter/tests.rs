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

    assert_eq!(
        *i.get("a".into()),
        Variable::Owned(ASTNode::Literal(Token::Number(1.)))
    );
    assert_eq!(
        *i.get("b".into()),
        Variable::Owned(ASTNode::Literal(Token::Bool(true)))
    );
    assert_eq!(
        *i.get("c".into()),
        Variable::Owned(ASTNode::Literal(Token::Str("str".into())))
    );
    assert_eq!(
        *i.get("d".into()),
        Variable::Owned(ASTNode::Literal(Token::Char('c')))
    );
}

#[test]
fn math() {
    let mut i = Interpreter::new();
    let ast = Parser::new(Lexer::new().lex(include_str!("tests/math.ly").to_string())).parse();
    i.execute(&ast);

    assert_eq!(
        *i.get("a".into()),
        Variable::Owned(ASTNode::Literal(Token::Number(2.)))
    );
    assert_eq!(
        *i.get("b".into()),
        Variable::Owned(ASTNode::Literal(Token::Number(0.)))
    );
    assert_eq!(
        *i.get("c".into()),
        Variable::Owned(ASTNode::Literal(Token::Number(25.)))
    );
    assert_eq!(
        *i.get("d".into()),
        Variable::Owned(ASTNode::Literal(Token::Number(2.)))
    );
    assert_eq!(
        *i.get("e".into()),
        Variable::Owned(ASTNode::Literal(Token::Number(6.)))
    );
}

#[test]
fn conditionals() {
    let mut i = Interpreter::new();
    let ast =
        Parser::new(Lexer::new().lex(include_str!("tests/conditionals.ly").to_string())).parse();
    i.execute(&ast);

    assert_eq!(
        *i.get("a".into()),
        Variable::Owned(ASTNode::Literal(Token::Number(5.)))
    );
}

#[test]
fn functions() {
    let mut i = Interpreter::new();
    let ast = Parser::new(Lexer::new().lex(include_str!("tests/functions.ly").to_string())).parse();
    i.execute(&ast);

    assert_eq!(
        *i.get("a".into()),
        Variable::Owned(ASTNode::Literal(Token::Number(10.)))
    );
    assert_eq!(
        *i.get("b".into()),
        Variable::Owned(ASTNode::Literal(Token::Number(20.)))
    );
    assert_eq!(
        *i.get("c".into()),
        Variable::Owned(ASTNode::Literal(Token::Bool(true)))
    );
}

#[test]
fn fibonacci() {
    let mut i = Interpreter::new();
    let ast = Parser::new(Lexer::new().lex(include_str!("tests/fibonacci.ly").to_string())).parse();
    i.execute(&ast);

    assert_eq!(
        *i.get("result".into()),
        Variable::Owned(ASTNode::Literal(Token::Number(21.)))
    );
}

#[test]
fn loops() {
    let mut i = Interpreter::new();
    let ast = Parser::new(Lexer::new().lex(include_str!("tests/loops.ly").to_string())).parse();
    i.execute(&ast);

    assert_eq!(
        *i.get("i".into()),
        Variable::Owned(ASTNode::Literal(Token::Number(25.)))
    );
    assert_eq!(
        *i.get("a".into()),
        Variable::Owned(ASTNode::Literal(Token::Number(25.)))
    );
}

