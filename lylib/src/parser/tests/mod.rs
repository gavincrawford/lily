#![cfg(test)]

use crate::{lexer::Token::*, parser::*, *};

/// Shorthand for creating and executing the parser, and comparing its output to an expression.
#[macro_export]
macro_rules! parse_eq {
    ($code:expr; $($block:expr),*) => {
        (|| {
            let result = Parser::new(Lexer::new().lex($code.into()).unwrap()).parse();
            assert!(result.is_ok(), "Parser failed: {:?}", result);
            assert_eq!(result.unwrap(), block!($($block),*));
        })()
    };
    ($code:expr, $path:expr; $($block:expr),*) => {
        (|| {
            let mut parser = Parser::new(Lexer::new().lex($code.into()).unwrap());
            parser.set_pwd($path.into());
            let result = parser.parse();
            assert!(result.is_ok(), "Parser failed: {:?}", result);
            assert_eq!(result.unwrap(), block!($($block),*));
        })()
    };
}

#[test]
fn decl() {
    parse_eq!(
        "let number = -1; let boolean = true;";
        node!(declare number => lit!(-1)),
        node!(declare boolean => lit!(true))
    );
}

#[test]
fn lists() {
    parse_eq!(
        "let list = [0, false, 'a']; let value = list[0]; list[0] = 0; list.obj = 0;";
        node!(declare list => node!([lit!(0), lit!(false), lit!('a')])),
        node!(declare value => node!(index ident!("list"), 0)),
        node!(assign node!(index ident!("list"), 0) => lit!(0)),
        node!(assign ident!("list.obj") => lit!(0))
    );
}

#[test]
fn indices() {
    parse_eq!(
        "let a = list[1][2][3]; let b = (list[1])[2]; let c = list[(1 + 1)];";
        node!(declare a => node!(index node!(index node!(index ident!("list"), 1), 2), 3)),
        node!(declare b => node!(index node!(index ident!("list"), 1), 2)),
        node!(declare c => ASTNode::Index {
            target: ident!("list"),
            index: node!(op lit!(1), Add, lit!(1)),
        }.into())
    );
}

#[test]
fn math() {
    parse_eq!(
        "let a = (1 + 1) + (1 + 1); let b = 1 + 2 - 3 * 4 / 5;";
        node!(declare a => node!(op node!(op lit!(1), Add, lit!(1)), Add, node!(op lit!(1), Add, lit!(1)))),
        node!(declare b => node!(op lit!(1), Add, node!(op lit!(2), Sub, node!(op lit!(3), Mul, node!(op lit!(4), Div, lit!(5))))))
    );
}

#[test]
fn comparisons() {
    parse_eq!(
        "let a = 100 < 200; let b = 100 <= 200; let c = 200 > 100; let d = 200 >= 100; let e = !true; let f = true && false; let g = true || false;";
        node!(declare a => node!(op lit!(100), LogicalL, lit!(200))),
        node!(declare b => node!(op lit!(100), LogicalLe, lit!(200))),
        node!(declare c => node!(op lit!(200), LogicalG, lit!(100))),
        node!(declare d => node!(op lit!(200), LogicalGe, lit!(100))),
        node!(declare e => node!(op lit!(true), LogicalNot, lit!(Token::Undefined))),
        node!(declare f => node!(op lit!(true), LogicalAnd, lit!(false))),
        node!(declare g => node!(op lit!(true), LogicalOr, lit!(false)))
    );
}

#[test]
fn conditionals() {
    parse_eq!(
        "if 2 > 1 do; a = b; end;";
        node!(
            if node!(op lit!(2), LogicalG, lit!(1)) =>
                block!(node!(assign a => ident!("b")));
            else =>
                block!();
        )
    );
}

#[test]
fn arguments() {
    parse_eq!(
        "let a = function((1 + 1), false, \"string\", 'c', [1, 2, 3]);";
        node!(declare a => node!(function(
            node!(op lit!(1), Add, lit!(1)),
            lit!(false),
            lit!("string"),
            lit!('c'),
            node!([lit!(1), lit!(2), lit!(3)])
        )))
    );
}

#[test]
fn functions() {
    parse_eq!(
        "func math a b do; let x = a + b; let y = a - b; return x * y; end;";
        node!(func math(a, b) => block!(
            node!(declare x => node!(op ident!("a"), Add, ident!("b"))),
            node!(declare y => node!(op ident!("a"), Sub, ident!("b"))),
            node!(return node!(op ident!("x"), Mul, ident!("y")))
        ))
    );
}

#[test]
fn import() {
    parse_eq!(
        "import \"./module1.ly\" as mod1; let ten = mod1.mod2.add2(5, 5);",
        "src/parser/tests";
        node!(mod mod1 => block!(
            node!(mod mod2 => block!(
                node!(func add2(a, b) => block!(
                    node!(return node!(op ident!("a"), Add, ident!("b")))
                ))
            )),
            node!(func add1(a, b) => block!(
                node!(return node!(op ident!("a"), Add, ident!("b")))
            ))
        )),
        node!(declare ten => node!(mod1.mod2.add2(lit!(5), lit!(5))))
    );
}

#[test]
fn structs() {
    parse_eq!(
        "struct Number do; let value = 0; end; let instance = new Number();";
        node!(struct Number => block!(
            node!(declare value => lit!(0))
        )),
        node!(declare instance => node!(Number()))
    );
}
