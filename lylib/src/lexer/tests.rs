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
        let result = Lexer::default().lex($input.into());
        lex_eq!(@compare result, $($token),*);
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
    lex_eq!("let var1 = 1; let var2 = 2; let var1.var2 = 3;" =>
        Let,
        Identifier(intern!("var1")),
        Equal,
        Number(1.),
        Endl,
        Let,
        Identifier(intern!("var2")),
        Equal,
        Number(2.),
        Endl,
        Let,
        Identifier(intern!("var1")),
        Dot,
        Identifier(intern!("var2")),
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
fn unary() {
    lex_eq!("var++; var--; !var; !!var;" =>
        Increment,
        Identifier(intern!("var")),
        Endl,
        Decrement,
        Identifier(intern!("var")),
        Endl,
        LogicalNot,
        Identifier(intern!("var")),
        Endl,
        LogicalNot,
        LogicalNot,
        Identifier(intern!("var")),
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
    lex_eq!("func func_name a b do; return a + b; end; func_name(a, b)" =>
        Function,
        Identifier(intern!("func_name")),
        Identifier(intern!("a")),
        Identifier(intern!("b")),
        BlockStart,
        Endl,
        Return,
        Identifier(intern!("a")),
        Add,
        Identifier(intern!("b")),
        Endl,
        BlockEnd,
        Endl,
        Identifier(intern!("func_name")),
        ParenOpen,
        Identifier(intern!("a")),
        Comma,
        Identifier(intern!("b")),
        ParenClose
    );

    lex_eq!("function(1 + 2, 3 + 4)" =>
        Identifier(intern!("function")),
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

    lex_eq!("fna(fnb())" =>
        Identifier(intern!("fna")),
        ParenOpen,
        Identifier(intern!("fnb")),
        ParenOpen,
        ParenClose,
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
    lex_eq!("'a' 'b' 'c' '\"';" =>
        Char('a'), Char('b'), Char('c'), Char('\"'), Endl
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
    lex_eq!("let list = [0, false, 'a']; let value = list[0];" =>
        Let,
        Identifier(intern!("list")),
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
        Identifier(intern!("value")),
        Equal,
        Identifier(intern!("list")),
        BracketOpen,
        Number(0.),
        BracketClose,
        Endl
    );
}

#[test]
fn loops() {
    lex_eq!("while true do; break; end;" =>
        While, Bool(true), BlockStart, Endl, Break, Endl, BlockEnd, Endl
    );
}

#[test]
fn modules() {
    lex_eq!("import \"./module.ly\"; import \"./module.ly\" as alias;" =>
        Import,
        Str("./module.ly".into()),
        Endl,
        Import,
        Str("./module.ly".into()),
        As,
        Identifier(intern!("alias")),
        Endl
    );
}

#[test]
fn structs() {
    lex_eq!("struct Number do; let value = 0; end; let instance = new Number;" =>
        Struct,
        Identifier(intern!("Number")),
        BlockStart,
        Endl,
        Let,
        Identifier(intern!("value")),
        Equal,
        Number(0.),
        Endl,
        BlockEnd,
        Endl,
        Let,
        Identifier(intern!("instance")),
        Equal,
        New,
        Identifier(intern!("Number")),
        Endl
    );
}

#[test]
fn tagged_spans() {
    // source:  l e t   x   =   4 2 ;
    // index:   0 1 2 3 4 5 6 7 8 9 10
    let src = "let x = 42;";
    let tagged = Lexer::default().lex_tagged(src.into()).unwrap();
    let kinds: Vec<&Token> = tagged.iter().map(|t| t.kind()).collect();
    assert_eq!(
        kinds,
        vec![&Let, &Identifier(intern!("x")), &Equal, &Number(42.), &Endl]
    );

    let spans: Vec<(usize, usize)> = tagged.iter().map(|t| (t.start(), t.end())).collect();
    assert_eq!(spans, vec![(0, 3), (4, 5), (6, 7), (8, 10), (10, 11)]);

    // and the slices must round-trip back to the source lexemes
    let slices: Vec<&str> = tagged.iter().map(|t| &src[t.start()..t.end()]).collect();
    assert_eq!(slices, vec!["let", "x", "=", "42", ";"]);
}

#[test]
fn tagged_spans_operators() {
    let src = "a == b;";
    let tagged = Lexer::default().lex_tagged(src.into()).unwrap();
    let eq_token = tagged.iter().find(|t| *t.kind() == LogicalEq).unwrap();
    assert_eq!(&src[eq_token.start()..eq_token.end()], "==");

    let src = "x = y;";
    let tagged = Lexer::default().lex_tagged(src.into()).unwrap();
    let single_eq = tagged.iter().find(|t| *t.kind() == Equal).unwrap();
    assert_eq!(&src[single_eq.start()..single_eq.end()], "=");
}

#[test]
fn tagged_lines_with_spans() {
    let src = "let x = 1\nlet y = 2;";
    let tagged = Lexer::default().lex_tagged(src.into()).unwrap();
    let lines: Vec<usize> = tagged.iter().map(|t| t.line()).collect();
    // line 1: let, x, =, 1, Endl (5);   line 2: let, y, =, 2, Endl (5)
    assert_eq!(lines, vec![1, 1, 1, 1, 1, 2, 2, 2, 2, 2]);

    // and the second-line tokens point at offsets past the newline
    let y_token = tagged
        .iter()
        .find(|t| *t.kind() == Identifier(intern!("y")))
        .unwrap();
    assert_eq!(&src[y_token.start()..y_token.end()], "y");
    assert_eq!(y_token.line(), 2);
}
