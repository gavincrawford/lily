#![cfg(test)]

use super::*;
use Token::*;

#[test]
fn variable_assignment() {
    assert_eq!(
        Lexer::new().lex("let x".into()),
        vec![Let, Identifier("x".into()), Endl]
    );
}

#[test]
fn math() {
    assert_eq!(
        Lexer::new().lex("1 + 1".into()),
        vec![Number(1.), Add, Number(1.), Endl]
    );
    assert_eq!(
        Lexer::new().lex("1 - 1".into()),
        vec![Number(1.), Sub, Number(1.), Endl]
    );
    assert_eq!(
        Lexer::new().lex("1 * 1".into()),
        vec![Number(1.), Mul, Number(1.), Endl]
    );
    assert_eq!(
        Lexer::new().lex("1 / 1".into()),
        vec![Number(1.), Div, Number(1.), Endl]
    );
}

#[test]
fn logic() {
    assert_eq!(
        Lexer::new().lex("1 == 2".into()),
        vec![Number(1.), LogicalEq, Number(2.), Endl]
    );
    assert_eq!(
        Lexer::new().lex("1 != 2".into()),
        vec![Number(1.), LogicalNeq, Number(2.), Endl]
    );
    assert_eq!(
        Lexer::new().lex("1 > 2".into()),
        vec![Number(1.), LogicalG, Number(2.), Endl]
    );
    assert_eq!(
        Lexer::new().lex("1 < 2".into()),
        vec![Number(1.), LogicalL, Number(2.), Endl]
    );
}

#[test]
fn conditionals() {
    assert_eq!(
        // TODO fix. this test SHOULD pass, but the lexer does not recognize the brackets as
        // separate characters, and instead converts them into an indentifier
        Lexer::new().lex("if 1 > 2 {}".into()),
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
}
