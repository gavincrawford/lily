//! Lexer tests.
//! Not all of these tests are syntax-compliant, but that's fine for this stage of the interpreter.
//! Here, we just want to make sure that plain text gets converted to the correct tokens.

#![cfg(test)]

use super::*;
use Token::*;

/// Shorthand for lexing input and comparing against expected tokens.
///
/// # Examples
///
/// ```rust
/// # use lylib::*;
/// # use lylib::lexer::Token::*;
/// lex_eq!("let x = 5;" => i => Let, Identifier(i.intern("x")), Equal, Number(5.), Endl);
/// // or...
/// lex_eq!("1 + 2;" => i => Number(1.), Add, Number(2.), Endl);
/// ```
#[macro_export]
macro_rules! lex_eq {
    ($input:expr => $($token:expr),*) => {
        let result = Lexer::new().lex($input.into());
        lex_eq!(@compare result, $($token),*);
    };
    ($input:expr => $interner:ident => $($token:expr),*) => {
        let result = Lexer::new().lex($input.into());
        let mut $interner = crate::get_global_interner().unwrap();
        lex_eq!(@compare result, $($token),*);
        drop($interner);
    };
    (@compare $result:expr, $($token:expr),*) => {
        assert!($result.is_ok(), "Lexer failed: {:?}", $result);
        assert_eq!(
            $result.unwrap(),
            vec![$($token),*]
        );
    }
}

#[test]
fn variable_assignment() {
    lex_eq!("let var1 = 1; let var2 = 2; let var1.var2 = 3;" => i =>
        Let,
        Identifier(i.intern("var1")),
        Equal,
        Number(1.),
        Endl,
        Let,
        Identifier(i.intern("var2")),
        Equal,
        Number(2.),
        Endl,
        Let,
        Identifier(i.intern("var1")),
        Dot,
        Identifier(i.intern("var2")),
        Equal,
        Number(3.),
        Endl
    );
}

#[test]
fn math() {
    lex_eq!("1+1+1+1;" =>
        Number(1.),
        Add,
        Number(1.),
        Add,
        Number(1.),
        Add,
        Number(1.),
        Endl
    );

    lex_eq!("32+12-7;" =>
        Number(32.), Add, Number(12.), Sub, Number(7.), Endl
    );

    lex_eq!("0.5731 * 0.222 / 1^3 // 10;" =>
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
    );
}

#[test]
fn logic() {
    lex_eq!("1 == 2; 1 != 2;" =>
        Number(1.),
        LogicalEq,
        Number(2.),
        Endl,
        Number(1.),
        LogicalNeq,
        Number(2.),
        Endl
    );

    lex_eq!("1 > 2 >= 3; 1 < 2 <= 3;" =>
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
    );

    lex_eq!("true && false; true || false;" =>
        Bool(true),
        LogicalAnd,
        Bool(false),
        Endl,
        Bool(true),
        LogicalOr,
        Bool(false),
        Endl
    );
}

#[test]
fn conditionals() {
    lex_eq!("if 1 > 2 do end;" =>
        If,
        Number(1.),
        LogicalG,
        Number(2.),
        BlockStart,
        BlockEnd,
        Endl
    );

    lex_eq!("if 1 < 2 do; 1 + 1; end;" =>
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
    );

    lex_eq!("if !true do end;" =>
        If, LogicalNot, Bool(true), BlockStart, BlockEnd, Endl
    );
}

#[test]
fn functions() {
    lex_eq!("func func_name a b do; return a + b; end; func_name(a, b)" => i =>
        Function,
        Identifier(i.intern("func_name")),
        Identifier(i.intern("a")),
        Identifier(i.intern("b")),
        BlockStart,
        Endl,
        Return,
        Identifier(i.intern("a")),
        Add,
        Identifier(i.intern("b")),
        Endl,
        BlockEnd,
        Endl,
        Identifier(i.intern("func_name")),
        ParenOpen,
        Identifier(i.intern("a")),
        Comma,
        Identifier(i.intern("b")),
        ParenClose
    );

    lex_eq!("function(1 + 2, 3 + 4)" => i =>
        Identifier(i.intern("function")),
        ParenOpen,
        Number(1.),
        Add,
        Number(2.),
        Comma,
        Number(3.),
        Add,
        Number(4.),
        ParenClose
    );
}

#[test]
fn strings() {
    lex_eq!("\"this is a string\";" =>
        Str("this is a string".into()), Endl
    );
}

#[test]
fn chars() {
    lex_eq!("'a' 'b' 'c';" =>
        Char('a'), Char('b'), Char('c'), Endl
    );
}

#[test]
fn bools() {
    lex_eq!("true false;" =>
        Bool(true), Bool(false), Endl
    );
}

#[test]
fn parens() {
    lex_eq!("(1 + 1) + 1;" =>
        ParenOpen,
        Number(1.),
        Add,
        Number(1.),
        ParenClose,
        Add,
        Number(1.),
        Endl
    );
}

#[test]
fn lists() {
    lex_eq!("let list = [0, false, 'a']; let value = list[0];" => i =>
        Let,
        Identifier(i.intern("list")),
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
        Identifier(i.intern("value")),
        Equal,
        Identifier(i.intern("list")),
        BracketOpen,
        Number(0.),
        BracketClose,
        Endl
    );
}

#[test]
fn loops() {
    lex_eq!("while true do; end;" =>
        While, Bool(true), BlockStart, Endl, BlockEnd, Endl
    );
}

#[test]
fn modules() {
    lex_eq!("import \"./module.ly\"; import \"./module.ly\" as alias;" => i =>
        Import,
        Str("./module.ly".into()),
        Endl,
        Import,
        Str("./module.ly".into()),
        As,
        Identifier(i.intern("alias")),
        Endl
    );
}

#[test]
fn structs() {
    lex_eq!("struct Number do; let value = 0; end; let instance = new Number;" => i =>
        Struct,
        Identifier(i.intern("Number")),
        BlockStart,
        Endl,
        Let,
        Identifier(i.intern("value")),
        Equal,
        Number(0.),
        Endl,
        BlockEnd,
        Endl,
        Let,
        Identifier(i.intern("instance")),
        Equal,
        New,
        Identifier(i.intern("Number")),
        Endl
    );
}
