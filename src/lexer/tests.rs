//! Lexer tests.
//! Not all of these tests are syntax-compliant, but that's fine for this stage of the interpreter.
//! Here, we just want to make sure that plain text gets converted to the correct tokens.

#![cfg(test)]

use super::*;
use Token::*;

#[test]
fn variable_assignment() {
    assert_eq!(
        Lexer::new().lex("let x;".into()),
        vec![Let, Identifier("x".into()), Endl]
    );
}

#[test]
fn math() {
    assert_eq!(
        Lexer::new().lex("1+1+1+1;".into()),
        vec![
            Number(1.),
            Add,
            Number(1.),
            Add,
            Number(1.),
            Add,
            Number(1.),
            Endl
        ]
    );
    assert_eq!(
        Lexer::new().lex("32+12-7;".into()),
        vec![Number(32.), Add, Number(12.), Sub, Number(7.), Endl]
    );
    assert_eq!(
        Lexer::new().lex("0.5731 * 0.222;".into()),
        vec![Number(0.5731), Mul, Number(0.222), Endl]
    );
    assert_eq!(
        Lexer::new().lex("1 / 1;".into()),
        vec![Number(1.), Div, Number(1.), Endl]
    );
}

#[test]
fn logic() {
    assert_eq!(
        Lexer::new().lex("1 == 2;".into()),
        vec![Number(1.), LogicalEq, Number(2.), Endl]
    );
    assert_eq!(
        Lexer::new().lex("1 != 2;".into()),
        vec![Number(1.), LogicalNeq, Number(2.), Endl]
    );
    assert_eq!(
        Lexer::new().lex("1 > 2 >= 3;".into()),
        vec![
            Number(1.),
            LogicalG,
            Number(2.),
            LogicalGe,
            Number(3.),
            Endl
        ]
    );
    assert_eq!(
        Lexer::new().lex("1 < 2 <= 3;".into()),
        vec![
            Number(1.),
            LogicalL,
            Number(2.),
            LogicalLe,
            Number(3.),
            Endl
        ]
    );
}

#[test]
fn conditionals() {
    assert_eq!(
        Lexer::new().lex("if 1 > 2 do end;".into()),
        vec![
            If,
            Number(1.),
            LogicalG,
            Number(2.),
            BlockStart,
            BlockEnd,
            Endl
        ]
    );
    assert_eq!(
        Lexer::new().lex("if 1 < 2 do; 1 + 1; end;".into()),
        vec![
            If,
            Number(1.),
            LogicalL,
            Number(2.),
            BlockStart,
            Endl,
            Number(1.),
            Add,
            Number(1.),
            Endl,
            BlockEnd,
            Endl
        ]
    );
}

#[test]
fn functions() {
    assert_eq!(
        Lexer::new().lex("func fn do; end;".into()),
        vec![
            Function,
            Identifier("fn".into()),
            BlockStart,
            Endl,
            BlockEnd,
            Endl,
        ]
    );
}

#[test]
fn strings() {
    assert_eq!(
        Lexer::new().lex("\"this is a string\";".into()),
        vec![Str(String::from("this is a string")), Endl,]
    )
}

#[test]
fn chars() {
    assert_eq!(
        Lexer::new().lex("'a' 'b' 'c';".into()),
        vec![Char('a'), Char('b'), Char('c'), Endl,]
    )
}
