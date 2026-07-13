//! Tests for the lexer that confirm that all span details, including line, start and end char, and
//! token type are all tracked correctly.

use super::*;

#[test]
fn spans_round_trip() {
    // source:  l e t   x   =   4 2 ;
    // index:   0 1 2 3 4 5 6 7 8 9 10
    let src = "let x = 42;";
    let spanned = Lexer::default().lex_spanned(src.into()).unwrap();
    let kinds: Vec<&Token> = spanned.iter().map(|t| t.kind()).collect();
    assert_eq!(
        kinds,
        vec![&Let, &Identifier(intern!("x")), &Equal, &Number(42.), &Endl]
    );

    let spans: Vec<(usize, usize)> = spanned.iter().map(|t| (t.start(), t.end())).collect();
    assert_eq!(spans, vec![(0, 3), (4, 5), (6, 7), (8, 10), (10, 11)]);

    // and the slices must round-trip back to the source lexemes
    let slices: Vec<&str> = spanned.iter().map(|t| &src[t.start()..t.end()]).collect();
    assert_eq!(slices, vec!["let", "x", "=", "42", ";"]);
}

#[test]
fn operator_spans() {
    let src = "a == b;";
    let spanned = Lexer::default().lex_spanned(src.into()).unwrap();
    let eq_token = spanned.iter().find(|t| *t.kind() == LogicalEq).unwrap();
    assert_eq!(&src[eq_token.start()..eq_token.end()], "==");

    let src = "x = y;";
    let spanned = Lexer::default().lex_spanned(src.into()).unwrap();
    let single_eq = spanned.iter().find(|t| *t.kind() == Equal).unwrap();
    assert_eq!(&src[single_eq.start()..single_eq.end()], "=");

    let src = "a += 1;";
    let spanned = Lexer::default().lex_spanned(src.into()).unwrap();
    let token = spanned.iter().find(|t| *t.kind() == AddAssign).unwrap();
    assert_eq!(&src[token.start()..token.end()], "+=");

    let src = "a -= 1;";
    let spanned = Lexer::default().lex_spanned(src.into()).unwrap();
    let token = spanned.iter().find(|t| *t.kind() == SubAssign).unwrap();
    assert_eq!(&src[token.start()..token.end()], "-=");

    let src = "a *= 1;";
    let spanned = Lexer::default().lex_spanned(src.into()).unwrap();
    let token = spanned.iter().find(|t| *t.kind() == MulAssign).unwrap();
    assert_eq!(&src[token.start()..token.end()], "*=");

    let src = "a /= 1;";
    let spanned = Lexer::default().lex_spanned(src.into()).unwrap();
    let token = spanned.iter().find(|t| *t.kind() == DivAssign).unwrap();
    assert_eq!(&src[token.start()..token.end()], "/=");
}

#[test]
fn multiline_spans() {
    let src = "let x = 1\nlet y = 2;";
    let spanned = Lexer::default().lex_spanned(src.into()).unwrap();
    let lines: Vec<usize> = spanned.iter().map(|t| t.line()).collect();
    // line 1: let, x, =, 1, Endl (5);   line 2: let, y, =, 2, Endl (5)
    assert_eq!(lines, vec![1, 1, 1, 1, 1, 2, 2, 2, 2, 2]);

    // and the second-line tokens point at offsets past the newline
    let y_token = spanned
        .iter()
        .find(|t| *t.kind() == Identifier(intern!("y")))
        .unwrap();
    assert_eq!(&src[y_token.start()..y_token.end()], "y");
    assert_eq!(y_token.line(), 2);
}

#[test]
fn comment_spans() {
    // a comment's newline must still advance the line counter
    let src = "# comment\nlet x = 1;";
    let spanned = Lexer::default().lex_spanned(src.into()).unwrap();
    let lines: Vec<usize> = spanned.iter().map(|t| t.line()).collect();
    // line 1: Endl (from the comment);   line 2: let, x, =, 1, Endl
    assert_eq!(lines, vec![1, 2, 2, 2, 2, 2]);

    // multiple consecutive comment lines should each advance the counter by one
    let src = "# one\n# two\n# three\nlet x = 1;";
    let spanned = Lexer::default().lex_spanned(src.into()).unwrap();
    let x_token = spanned
        .iter()
        .find(|t| *t.kind() == Identifier(intern!("x")))
        .unwrap();
    assert_eq!(x_token.line(), 4);
}
