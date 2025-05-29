#![cfg(test)]

use super::*;
use crate::lexer::Lexer;

// expose utility macros
#[macro_use]
mod macros;

// expose token variants
use Token::*;

#[test]
fn decl() {
    let result = parse!("let number = -1; let boolean = true;");
    assert_eq!(
        result.unwrap(),
        node!(block vec![
            node!(declare ident!("number") => lit!(-1)),
            node!(declare ident!("boolean") => lit!(Bool(true))),
        ])
    );
}

#[test]
fn lists() {
    let result = parse!("let list = [0, false, 'a']; let value = list[0];");
    assert_eq!(
        result.unwrap(),
        node!(block vec![
            node!(declare ident!("list") => node!([lit!(0), lit!(Bool(false)), lit!(Char('a'))])),
            node!(declare ident!("value") => node!(list[0])),
        ])
    );
}

#[test]
fn math() {
    let result = parse!("let x = 1 + 2 - 3 * 4 / 5;");
    assert_eq!(
        result.unwrap(),
        node!(block vec![node!(declare ident!("x") =>
            node!(op lit!(1), Add, node!(op lit!(2), Sub, node!(op lit!(3), Mul, node!(op lit!(4), Div, lit!(5)))))
        )])
    );
}

#[test]
fn comparisons() {
    let result =
        parse!("let a = 100 < 200; let b = 100 <= 200; let c = 200 > 100; let d = 200 >= 100;");
    assert_eq!(
        result.unwrap(),
        node!(block vec![
            node!(declare ident!("a") => node!(op lit!(100), LogicalL, lit!(200))),
            node!(declare ident!("b") => node!(op lit!(100), LogicalLe, lit!(200))),
            node!(declare ident!("c") => node!(op lit!(200), LogicalG, lit!(100))),
            node!(declare ident!("d") => node!(op lit!(200), LogicalGe, lit!(100))),
        ])
    );
}

#[test]
fn conditionals() {
    let result = parse!("if 2 > 1 do; a = b; end;");
    assert_eq!(
        result.unwrap(),
        node!(block vec![
            node!(
                if node!(op lit!(2), LogicalG, lit!(1)) =>
                    node!(block vec![node!(assign ident!("a") => ident!("b"))]);
                else =>
                    node!(block vec![]);
            )
        ])
    );
}

#[test]
fn arguments() {
    let result = parse!("let result = function((1 + 1) * 2)");
    assert_eq!(
        result.unwrap(),
        node!(block vec![
            node!(declare ident!("result") => node!(function(node!(op node!(op lit!(1), Add, lit!(1)), Mul, lit!(2)))))
        ])
    );
}

#[test]
fn functions() {
    let result = parse!("func math a b do; let x = a + b; let y = a - b; return x * y; end;");
    assert_eq!(
        result.unwrap(),
        node!(block vec![
            node!(func math(a, b) => node!(block vec![
                node!(declare ident!("x") => node!(op ident!("a"), Add, ident!("b"))),
                node!(declare ident!("y") => node!(op ident!("a"), Sub, ident!("b"))),
                node!(return node!(op ident!("x"), Mul, ident!("y"))),
            ]))
        ])
    );
}

#[test]
fn import() {
    let result = parse!(
        "import \"./module1.ly\" as mod1; let ten = mod1.mod2.add2(5, 5);",
        "src/parser/tests"
    );
    assert_eq!(
        result.unwrap(),
        node!(block vec![
            node!(mod mod1 => node!(block vec![
                node!(mod mod2 => node!(block vec![
                    node!(func add2(a, b) => node!(block vec![
                        node!(return node!(op ident!("a"), Add, ident!("b")))
                    ]))
                ])),
                node!(func add1(a, b) => node!(block vec![
                    node!(return node!(op ident!("a"), Add, ident!("b")))
                ]))
            ])),
            node!(declare ident!("ten") => node!(mod1.mod2.add2(lit!(5), lit!(5))))
        ])
    );
}

#[test]
fn structs() {
    let result = parse!("struct Number do; let value = 0; end; let instance = new Number();");
    assert_eq!(
        result.unwrap(),
        node!(block vec![
            node!(struct Number => node!(block vec![
                node!(declare ident!("value") => lit!(0))
            ])),
            node!(declare ident!("instance") => node!(Number()))
        ])
    );
}
