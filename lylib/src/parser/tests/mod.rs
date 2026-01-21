#![cfg(test)]

use crate::{lexer::Token::*, parser::*, *};

/// Shorthand for creating and executing the parser, and comparing its output to an expression.
#[macro_export]
macro_rules! parse_eq {
    ($code:expr; $($block:expr),*) => {{
        let result = Parser::new(Lexer::default().lex($code.into()).unwrap()).parse();
        assert!(result.is_ok(), "Parser failed: {:?}", result);
        let result = result.unwrap(); // safety ^^^
        let block = block!($($block),*);
        if result != block {
            panic!("expected: {:#?}\ngot: {:#?}", block, result);
        }
    }};
    ($code:expr, $path:expr; $($block:expr),*) => {{
        let mut parser = Parser::new(Lexer::default().lex($code.into()).unwrap());
        parser.set_pwd($path.into());
        let result = parser.parse();
        assert!(result.is_ok(), "Parser failed: {:?}", result);
        let result = result.unwrap(); // safety ^^^
        let block = block!($($block),*);
        if result != block {
            panic!("expected: {:#?}\ngot: {:#?}", block, result);
        }
    }};
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
fn derefs() {
    parse_eq!(
        "a.b; a().b; a().b().c;";
        node!(a.b),
        node!(deref node!(a()), ident!("b")),
        node!(deref node!(call node!(deref node!(a()), ident!("b"))), ident!("c"))
    );
}

#[test]
fn lists() {
    parse_eq!(
        "let list = [0, false, 'a']; let value = list[0]; list[0] = 0; list.obj = 0;";
        node!(declare list => node!([lit!(0), lit!(false), lit!('a')])),
        node!(declare value => node!(index ident!("list"), 0)),
        node!(assign node!(index ident!("list"), 0) => lit!(0)),
        node!(assign node!(list.obj) => lit!(0))
    );
}

#[test]
fn indices() {
    parse_eq!(
        "let a = list[1][2][3]; let b = (list[1])[2];";
        node!(declare a => node!(index node!(index node!(index ident!("list"), 1), 2), 3)),
        node!(declare b => node!(index node!(index ident!("list"), 1), 2))
    );
}

#[test]
fn indices_complex() {
    parse_eq!(
        "let a = x[(1 + 1)]; let b = x[y[0]]; let c = x[y[z][0]];";
        node!(declare a => node!(x[node!(op 1, Add, 1)])),
        node!(declare b => node!(x[node!(y[0])])),
        node!(declare c => node!(x[node!(index node!(y[ident!("z")]), 0)]))
    );
}

#[test]
fn math() {
    parse_eq!(
        "let a = (1 + 1) + (1 + 1); let b = 1 + 2 - 3 * 4 / 5;";
        node!(declare a => node!(op node!(op 1, Add, 1), Add, node!(op 1, Add, 1))),
        node!(declare b => node!(op lit!(1), Add, node!(op lit!(2), Sub, node!(op node!(op 3, Mul, 4), Div, lit!(5)))))
    );
}

#[test]
fn math_complex() {
    parse_eq!(
        "let a = (1 + (2 / 4)) + (((((1))+((1)))));";
        node!(declare a => node!(op node!(op lit!(1), Add, node!(op 2, Div, 4)), Add, node!(op 1, Add, 1)))
    );
}

#[test]
fn comparisons() {
    parse_eq!(
        "let a = 100 < 200;
        let b = 100 <= 200;
        let c = 200 > 100;
        let d = 200 >= 100;
        let e = true && false;
        let f = true || false;
        a++;
        a--;";
        node!(declare a => node!(op 100, LogicalL, 200)),
        node!(declare b => node!(op 100, LogicalLe, 200)),
        node!(declare c => node!(op 200, LogicalG, 100)),
        node!(declare d => node!(op 200, LogicalGe, 100)),
        node!(declare e => node!(op true, LogicalAnd, false)),
        node!(declare f => node!(op true, LogicalOr, false)),
        node!(unary Increment, ident!("a")),
        node!(unary Decrement, ident!("a"))
    );
}

#[test]
fn unary() {
    parse_eq!(
        "let a = !true;
        let b = !!true;
        let c = !!!true;";
        node!(declare a => node!(unary LogicalNot, lit!(true))),
        node!(declare b => node!(unary LogicalNot, node!(unary LogicalNot, lit!(true)))),
        node!(declare c => node!(unary LogicalNot, node!(unary LogicalNot, node!(unary LogicalNot, lit!(true)))))
    )
}

#[test]
fn unary_complex() {
    parse_eq!(
        "let a = -(1 + 2);
        let b = -x;
        let c = !!(1 + 1);
        let d = -list[0];";
        node!(declare a => node!(unary Sub, node!(op 1, Add, 2))),
        node!(declare b => node!(unary Sub, ident!("x"))),
        node!(declare c => node!(unary LogicalNot, node!(unary LogicalNot, node!(op 1, Add, 1)))),
        node!(declare d => node!(unary Sub, node!(list[0])))
    );
}

#[test]
fn unary_mixed() {
    parse_eq!(
        "let a = -!true;
        let b = !-x;
        let c = -!!y;";
        node!(declare a => node!(unary Sub, node!(unary LogicalNot, lit!(true)))),
        node!(declare b => node!(unary LogicalNot, node!(unary Sub, ident!("x")))),
        node!(declare c => node!(unary Sub, node!(unary LogicalNot, node!(unary LogicalNot, ident!("y")))))
    );
}

#[test]
fn nested_imports() {
    parse_eq!(
        "import \"./module1.ly\" as mod1; let ten_mod1 = mod1.add1(5, 5); let ten_mod2 = mod1.mod2.add2(5, 5);",
        "src/parser/tests/nested_imports";
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
        node!(declare ten_mod1 => node!(mod1.add1(lit!(5), lit!(5)))),
        node!(declare ten_mod2 => node!(mod1.mod2.add2(lit!(5), lit!(5))))
    );
}

#[test]
fn precedence() {
    parse_eq!(
        "let a = 1 + 1 == 4 / 2;
        let b = 2 * 3 + 4 * 5;
        let c = 2 + 3 * 4 + 5;
        let d = true && false || true;
        let e = 1 < 2 && 3 > 2;
        let f = 2 ^ 3 * 4;
        let g = a.x + b.y;";

        // Test that comparison has lower precedence than arithmetic
        node!(declare a => node!(op node!(op 1, Add, 1), LogicalEq, node!(op 4, Div, 2))),

        // Test that multiplication has higher precedence than addition
        node!(declare b => node!(op node!(op 2, Mul, 3), Add, node!(op 4, Mul, 5))),

        // Test mixed precedence
        node!(declare c => node!(op lit!(2), Add, node!(op node!(op 3, Mul, 4), Add, lit!(5)))),

        // Logical AND has higher precedence than OR
        node!(declare d => node!(op node!(op true, LogicalAnd, false), LogicalOr, lit!(true))),

        // Comparisons have higher precedence than logical AND
        node!(declare e => node!(op node!(op 1, LogicalL, 2), LogicalAnd, node!(op 3, LogicalG, 2))),

        // Power & deref have highest precedence
        node!(declare f => node!(op node!(op 2, Pow, 3), Mul, lit!(4))),
        node!(declare g => node!(op node!(a.x), Add, node!(b.y)))
    );
}

#[test]
fn conditionals() {
    parse_eq!(
        "if 2 > 1 do; a = b; end;
        if 1 do; end;
        if 1 + 1 > 2 do; end;
        if true do; if true do; end; end;";
        node!(
            if node!(op 2, LogicalG, 1) =>
                block!(node!(assign a => ident!("b")));
            else =>
                block!();
        ),
        node!(
            if lit!(1) =>
                block!();
            else =>
                block!();
        ),
        node!(
            if node!(op node!(op 1, Add, 1), LogicalG, lit!(2)) =>
                block!();
            else =>
                block!();
        ),
        node!(
            if lit!(true) =>
                block!(node!(
                    if lit!(true) =>
                        block!();
                    else =>
                        block!();
                ));
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
            node!(op 1, Add, 1),
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
        "func math a b do; let x = a + b; let y = a - b; return x * y; end; let other = math;";
        node!(func math(a, b) => block!(
            node!(declare x => node!(op ident!("a"), Add, ident!("b"))),
            node!(declare y => node!(op ident!("a"), Sub, ident!("b"))),
            node!(return node!(op ident!("x"), Mul, ident!("y")))
        )),
        node!(declare other => ident!("math"))
    );
}

#[test]
fn function_calls() {
    parse_eq!(
        "a();";
        node!(a())
    );
    parse_eq!(
        "a().b();";
        node!(call node!(deref node!(a()), ident!("b")))
    );
    parse_eq!(
        "a().b().c();";
        node!(call node!(deref node!(call node!(deref node!(a()), ident!("b"))), ident!("c")))
    );
}

#[test]
fn structs() {
    parse_eq!(
        "struct Number; let value = 0; end; let instance = new Number();";
        node!(struct Number => block!(
            node!(declare value => lit!(0))
        )),
        node!(declare instance => node!(Number()))
    );
}

#[test]
fn loops() {
    parse_eq!(
        "while true do; a = a + 1; end;
        while x < 10 do; x++; end;
        while 1 + 1 > 0 do; break; end;";
        node!(
            loop lit!(true) =>
                block!(node!(assign a => node!(op ident!("a"), Add, lit!(1))));
        ),
        node!(
            loop node!(op ident!("x"), LogicalL, lit!(10)) =>
                block!(node!(unary Increment, ident!("x")));
        ),
        node!(
            loop node!(op node!(op 1, Add, 1), LogicalG, lit!(0)) =>
                block!(node!(break));
        )
    );
}
