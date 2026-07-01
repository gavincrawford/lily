#![cfg(test)]

use crate::{errors::ParserError, interpreter::AsID, lexer::Token::*, parser::*};

/// Shorthand for creating and executing the parser, and comparing its output to an expression.
macro_rules! parse_test {
    // Error variant: asserts parsing fails with a `ParserError` matching the given pattern
    // *somewhere* in the `anyhow::Error` cause chain.
    ($test:tt => $code:expr; error $pat:pat) => {
        #[test]
        fn $test() {
            let result = Parser::new(Lexer::default().lex_spanned($code.into()).unwrap()).unwrap().parse();
            match result {
                Ok(ast) => panic!(
                    "expected parser error matching `{}`, but parsing succeeded: {:#?}",
                    stringify!($pat),
                    ast
                ),
                Err(e) => {
                    let found = e.chain().any(|cause| matches!(cause.downcast_ref::<ParserError>(), Some($pat)));
                    assert!(
                        found,
                        "expected error matching `{}` in chain, got: {:?}",
                        stringify!($pat),
                        e
                    );
                }
            }
        }
    };

    // Test w/ modified parser path
    ($test:tt ($path:expr) => $code:expr; $($block:expr),*) => {
        #[test]
        fn $test() {
            let mut parser = Parser::new(Lexer::default().lex_spanned($code.into()).unwrap()).unwrap();
            parser.set_cwd($path.into());
            let result = parser.parse();
            assert!(result.is_ok(), "Parser failed: {:?}", result);
            let result = result.unwrap(); // safety ^^^
            let block = block!($($block),*);
            if result != block {
                panic!("expected: {:#?}\ngot: {:#?}", block, result);
            }
        }
    };

    // Test only
    ($test:tt => $code:expr; $($block:expr),*) => {
        #[test]
        fn $test() {
            let result = Parser::new(Lexer::default().lex_spanned($code.into()).unwrap()).unwrap().parse();
            assert!(result.is_ok(), "Parser failed: {:?}", result);
            let result = result.unwrap(); // safety ^^^
            let block = block!($($block),*);
            if result != block {
                panic!("expected: {:#?}\ngot: {:#?}", block, result);
            }
        }
    };
}

parse_test!(compound_assign =>
    "a += 1;
    a -= 1;
    a *= 1;
    a /= 1;
    x.y.z += 2;
    a += 1 + 2;";
    node!(assign a => node!(op ident!("a"), Add, lit!(1))),
    node!(assign a => node!(op ident!("a"), Sub, lit!(1))),
    node!(assign a => node!(op ident!("a"), Mul, lit!(1))),
    node!(assign a => node!(op ident!("a"), Div, lit!(1))),
    node!(assign node!(x.y.z) => node!(op node!(x.y.z), Add, lit!(2))),
    node!(assign a => node!(op ident!("a"), Add, node!(op 1, Add, 2)))
);

parse_test!(decl => "let number = -1; let boolean = true;";
    node!(declare number => lit!(-1)),
    node!(declare boolean => lit!(true))
);

parse_test!(decl_incomplete => "let var = ;"; error ParserError::Declaration(_));

parse_test!(derefs => "a.b; a().b; a().b().c;";
    node!(a.b),
    node!(deref node!(a()), ident!("b")),
    node!(deref node!(call node!(deref node!(a()), ident!("b"))), ident!("c"))
);

parse_test!(lists => "let list = [0, false, 'a']; let value = list[0]; list[0] = 0; list.obj = 0;";
    node!(declare list => node!([lit!(0), lit!(false), lit!('a')])),
    node!(declare value => node!(index ident!("list"), 0)),
    node!(assign node!(index ident!("list"), 0) => lit!(0)),
    node!(assign node!(list.obj) => lit!(0))
);

parse_test!(indices => "let a = list[1][2][3]; let b = (list[1])[2];";
    node!(declare a => node!(index node!(index node!(index ident!("list"), 1), 2), 3)),
    node!(declare b => node!(index node!(index ident!("list"), 1), 2))
);

parse_test!(indices_complex => "let a = x[(1 + 1)]; let b = x[y[0]]; let c = x[y[z][0]];";
    node!(declare a => node!(x[node!(op 1, Add, 1)])),
    node!(declare b => node!(x[node!(y[0])])),
    node!(declare c => node!(x[node!(index node!(y[ident!("z")]), 0)]))
);

parse_test!(math => "let a = (1 + 1) + (1 + 1); let b = 1 + 2 - 3 * 4 / 5;";
    node!(declare a => node!(op node!(op 1, Add, 1), Add, node!(op 1, Add, 1))),
    node!(declare b => node!(op lit!(1), Add, node!(op lit!(2), Sub, node!(op node!(op 3, Mul, 4), Div, lit!(5)))))
);

parse_test!(math_complex => "let a = (1 + (2 / 4)) + (((((1))+((1)))));";
    node!(declare a => node!(op node!(op lit!(1), Add, node!(op 2, Div, 4)), Add, node!(op 1, Add, 1)))
);

parse_test!(comparisons =>
    "let a = 100 < 200;
    let b = 100 <= 200;
    let c = 200 > 100;
    let d = 200 >= 100;
    let e = true && false;
    let f = true || false;";
    node!(declare a => node!(op 100, LogicalL, 200)),
    node!(declare b => node!(op 100, LogicalLe, 200)),
    node!(declare c => node!(op 200, LogicalG, 100)),
    node!(declare d => node!(op 200, LogicalGe, 100)),
    node!(declare e => node!(op true, LogicalAnd, false)),
    node!(declare f => node!(op true, LogicalOr, false))
);

parse_test!(comparison_incomplete => "let res = 100 < ;"; error ParserError::Declaration(_));

parse_test!(unclosed_paren => "let value = (1 + ;"; error ParserError::Declaration(_));

parse_test!(unclosed_bracket => "let list = [1, 2, 3, ;"; error ParserError::UnexpectedEOF);

parse_test!(unary =>
    "let a = !true;
    let b = !!true;
    let c = !!!true;";
    node!(declare a => node!(unary LogicalNot, lit!(true))),
    node!(declare b => node!(unary LogicalNot, node!(unary LogicalNot, lit!(true)))),
    node!(declare c => node!(unary LogicalNot, node!(unary LogicalNot, node!(unary LogicalNot, lit!(true)))))
);

parse_test!(unary_inc_dec =>
    "a++;
    a--;
    x.y.z++;";
    node!(assign a => node!(op ident!("a"), Add, lit!(1))),
    node!(assign a => node!(op ident!("a"), Sub, lit!(1))),
    node!(assign node!(x.y.z) => node!(op node!(x.y.z), Add, lit!(1)))
);

parse_test!(unary_inc_dec_in_expr =>
    "let y = 2 + x++;
    let z = x-- * 3;";
    node!(declare y => node!(op lit!(2), Add, node!(assign x => node!(op ident!("x"), Add, lit!(1))))),
    node!(declare z => node!(op node!(assign x => node!(op ident!("x"), Sub, lit!(1))), Mul, lit!(3)))
);

parse_test!(unary_complex =>
    "let a = -(1 + 2);
    let b = -x;
    let c = !!(1 + 1);
    let d = -list[0];";
    node!(declare a => node!(unary Sub, node!(op 1, Add, 2))),
    node!(declare b => node!(unary Sub, ident!("x"))),
    node!(declare c => node!(unary LogicalNot, node!(unary LogicalNot, node!(op 1, Add, 1)))),
    node!(declare d => node!(unary Sub, node!(list[0])))
);

parse_test!(unary_mixed =>
    "let a = -!true;
    let b = !-x;
    let c = -!!y;";
    node!(declare a => node!(unary Sub, node!(unary LogicalNot, lit!(true)))),
    node!(declare b => node!(unary LogicalNot, node!(unary Sub, ident!("x")))),
    node!(declare c => node!(unary Sub, node!(unary LogicalNot, node!(unary LogicalNot, ident!("y")))))
);

parse_test!(nested_imports ("src/parser/tests/nested_imports") =>
    "import \"./module1.ly\" as mod1; let ten_mod1 = mod1.add1(5, 5); let ten_mod2 = mod1.mod2.add2(5, 5);";
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

parse_test!(missing_import => "import \"./does_not_exist.ly\";"; error ParserError::Import(_));

parse_test!(precedence =>
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

parse_test!(conditionals =>
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

parse_test!(arguments => "let a = function((1 + 1), false, \"string\", 'c', [1, 2, 3]);";
    node!(declare a => node!(function(
        node!(op 1, Add, 1),
        lit!(false),
        lit!("string"),
        lit!('c'),
        node!([lit!(1), lit!(2), lit!(3)])
    )))
);

parse_test!(functions => "func math a b do; let x = a + b; let y = a - b; return x * y; end; let other = math;";
    node!(func math(a, b) => block!(
        node!(declare x => node!(op ident!("a"), Add, ident!("b"))),
        node!(declare y => node!(op ident!("a"), Sub, ident!("b"))),
        node!(return node!(op ident!("x"), Mul, ident!("y")))
    )),
    node!(declare other => ident!("math"))
);

parse_test!(function_call => "a(); b(); c();";
    node!(a()),
    node!(b()),
    node!(c())
);

parse_test!(function_call_nested => "a().b().c(); (a()).b().c(); (a().b()).c();";
    node!(call node!(deref node!(call node!(deref node!(a()), ident!("b"))), ident!("c"))),
    node!(call node!(deref node!(call node!(deref node!(a()), ident!("b"))), ident!("c"))),
    node!(call node!(deref node!(call node!(deref node!(a()), ident!("b"))), ident!("c")))
);

parse_test!(structs => "struct Number; let value = 0; end; let instance = new Number();";
    node!(struct Number => block!(
        node!(declare value => lit!(0))
    )),
    node!(declare instance => node!(Number()))
);

parse_test!(loops =>
    "while true do; a = a + 1; end;
    while x < 10 do; x++; end;
    while 1 + 1 > 0 do; break; end;";
    node!(
        loop lit!(true) =>
            block!(node!(assign a => node!(op ident!("a"), Add, lit!(1))));
    ),
    node!(
        loop node!(op ident!("x"), LogicalL, lit!(10)) =>
            block!(node!(assign x => node!(op ident!("x"), Add, lit!(1))));
    ),
    node!(
        loop node!(op node!(op 1, Add, 1), LogicalG, lit!(0)) =>
            block!(node!(break));
    )
);
