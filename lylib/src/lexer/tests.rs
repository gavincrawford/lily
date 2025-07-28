//! Lexer tests.
//! Not all of these tests are syntax-compliant, but that's fine for this stage of the interpreter.
//! Here, we just want to make sure that plain text gets converted to the correct tokens.

#![cfg(test)]

use super::*;
use Token::*;

// TODO write new macros to improve on lexer test clarity

#[test]
fn variable_assignment() {
    let result = Lexer::new().lex("let var1 = 1; let var2 = 2;".into());
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![
            Let,
            Identifier("var1".into()),
            Equal,
            Number(1.),
            Endl,
            Let,
            Identifier("var2".into()),
            Equal,
            Number(2.),
            Endl
        ]
    );
}

#[test]
fn math() {
    let result = Lexer::new().lex("1+1+1+1;".into());
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
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

    let result = Lexer::new().lex("32+12-7;".into());
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![Number(32.), Add, Number(12.), Sub, Number(7.), Endl]
    );

    let result = Lexer::new().lex("0.5731 * 0.222 / 1^3 // 10;".into());
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![
            Number(0.5731),
            Mul,
            Number(0.222),
            Div,
            Number(1.),
            Pow,
            Number(3.),
            Floor,
            Number(10.),
            Endl
        ]
    );
}

#[test]
fn logic() {
    let result = Lexer::new().lex("1 == 2; 1 != 2;".into());
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![
            Number(1.),
            LogicalEq,
            Number(2.),
            Endl,
            Number(1.),
            LogicalNeq,
            Number(2.),
            Endl
        ]
    );

    let result = Lexer::new().lex("1 > 2 >= 3; 1 < 2 <= 3;".into());
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![
            Number(1.),
            LogicalG,
            Number(2.),
            LogicalGe,
            Number(3.),
            Endl,
            Number(1.),
            LogicalL,
            Number(2.),
            LogicalLe,
            Number(3.),
            Endl
        ]
    );

    let result = Lexer::new().lex(" true && false; true || false;".into());
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![
            Bool(true),
            LogicalAnd,
            Bool(false),
            Endl,
            Bool(true),
            LogicalOr,
            Bool(false),
            Endl,
        ]
    );
}

#[test]
fn conditionals() {
    let result = Lexer::new().lex("if 1 > 2 do end;".into());
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
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

    let result = Lexer::new().lex("if 1 < 2 do; 1 + 1; end;".into());
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
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

    let result = Lexer::new().lex("if !true do end;".into());
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![If, LogicalNot, Bool(true), BlockStart, BlockEnd, Endl]
    );
}

#[test]
fn functions() {
    let result =
        Lexer::new().lex("func func_name a b do; return a + b; end; func_name(a, b)".into());
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![
            Function,
            Identifier("func_name".into()),
            Identifier("a".into()),
            Identifier("b".into()),
            BlockStart,
            Endl,
            Return,
            Identifier("a".into()),
            Add,
            Identifier("b".into()),
            Endl,
            BlockEnd,
            Endl,
            Identifier("func_name".into()),
            ParenOpen,
            Identifier("a".into()),
            Comma,
            Identifier("b".into()),
            ParenClose,
        ]
    );

    let result = Lexer::new().lex("function(1 + 2, 3 + 4)".into());
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![
            Identifier("function".into()),
            ParenOpen,
            Number(1.),
            Add,
            Number(2.),
            Comma,
            Number(3.),
            Add,
            Number(4.),
            ParenClose,
        ]
    );
}

#[test]
fn strings() {
    let result = Lexer::new().lex("\"this is a string\";".into());
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![Str(String::from("this is a string")), Endl,]
    );
}

#[test]
fn chars() {
    let result = Lexer::new().lex("'a' 'b' 'c';".into());
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![Char('a'), Char('b'), Char('c'), Endl,]
    );
}

#[test]
fn bools() {
    let result = Lexer::new().lex("true false;".into());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![Bool(true), Bool(false), Endl,]);
}

#[test]
fn parens() {
    let result = Lexer::new().lex("(1 + 1) + 1;".into());
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![
            ParenOpen,
            Number(1.),
            Add,
            Number(1.),
            ParenClose,
            Add,
            Number(1.),
            Endl,
        ]
    );
}

#[test]
fn lists() {
    let result = Lexer::new().lex("let list = [0, false, 'a']; let value = list[0];".into());
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![
            Let,
            Identifier("list".into()),
            Equal,
            BracketOpen,
            Number(0.),
            Comma,
            Bool(false),
            Comma,
            Char('a'),
            BracketClose,
            Endl,
            Let,
            Identifier("value".into()),
            Equal,
            Identifier("list".into()),
            BracketOpen,
            Number(0.),
            BracketClose,
            Endl,
        ]
    );
}

#[test]
fn loops() {
    let result = Lexer::new().lex("while true do; end;".into());
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![While, Bool(true), BlockStart, Endl, BlockEnd, Endl]
    );
}

#[test]
fn modules() {
    let result =
        Lexer::new().lex("import \"./module.ly\"; import \"./module.ly\" as alias;".into());
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![
            Import,
            Str("./module.ly".into()),
            Endl,
            Import,
            Str("./module.ly".into()),
            As,
            Identifier("alias".into()),
            Endl
        ]
    );
}

#[test]
fn structs() {
    let result =
        Lexer::new().lex("struct Number do; let value = 0; end; let instance = new Number;".into());
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![
            Struct,
            Identifier("Number".into()),
            BlockStart,
            Endl,
            Let,
            Identifier("value".into()),
            Equal,
            Number(0.),
            Endl,
            BlockEnd,
            Endl,
            Let,
            Identifier("instance".into()),
            Equal,
            New,
            Identifier("Number".into()),
            Endl,
        ]
    );
}
