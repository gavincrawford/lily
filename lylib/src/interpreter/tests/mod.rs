// allow unused b/c the rust lsp fails to recognize imports in submodules
#![allow(unused_macros, unused_imports)]

use crate::{
    interpreter::*,
    lexer::{Token::*, *},
    parser::{ASTNode::*, *},
    *,
};

/// Shorthand for comparing a variable with any owned value.
macro_rules! var_eq {
    ($interpreter:expr, $id:tt, $node:expr) => {
        let (got, expected) = (
            $interpreter.get(&stringify!($id).as_id()).unwrap(),
            Variable::Owned($node).into(),
        );
        if got != expected {
            panic!(
                "expected {} to be {expected:?}, found {got:?}.",
                stringify!($id)
            );
        }
    };
}

/// Shorthand for comparing a variable with an owned literal.
macro_rules! var_eq_literal {
    ($interpreter:expr, $id:tt, $token:expr) => {
        var_eq!($interpreter, $id, ASTNode::inner_to_owned(&lit!($token)));
    };
}

/// Expands into entire test cases. The name of the function provided is expected to be the name
/// of the test file, given the file extension is omitted.
/// # Example
/// ```ignore
/// test!(file_name => ( // will read `file_name.ly`
///     // use `:=` to implicitly create a literal
///     literal_string := "expected value",
///     // use `==` for custom nodes
///     other_node == node!([lit!(1), lit!(2)]),
/// ));
/// ```
macro_rules! test {
    // Helpers for progressively munching equality test statements.
    (@munch $interpreter:ident; $lhs:tt == $rhs:expr, $($rest:tt)*) => {
        var_eq!($interpreter, $lhs, $rhs);
        test!(@munch $interpreter; $($rest)*);
    };
    (@munch $interpreter:ident; $lhs:tt == $rhs:expr) => {
        var_eq!($interpreter, $lhs, $rhs);
    };
    (@munch $interpreter:ident; $lhs:tt := $rhs:expr, $($rest:tt)*) => {
        var_eq_literal!($interpreter, $lhs, $rhs);
        test!(@munch $interpreter; $($rest)*);
    };
    (@munch $interpreter:ident; $lhs:tt := $rhs:expr) => {
        var_eq_literal!($interpreter, $lhs, $rhs);
    };

    // Helper for running the file.
    (@interpret $path:expr) => {{
        // interpret file
        use std::io::Cursor;
        let mut i = Interpreter::new(Cursor::new(vec![]), Cursor::new(vec![]));
        let mut p = Parser::new(Lexer::new().lex(include_str!($path).to_string()).unwrap());
        p.set_pwd(std::path::PathBuf::from("src/interpreter/tests/feature/"));
        let ast = p.parse().unwrap();
        i.execute(ast).unwrap();

        // read output
        let mut buf = String::new();
        i.output().set_position(0);
        i.output().read_to_string(&mut buf).unwrap();

        (i, buf)
    }};

    // Test for variable equality
    ($file:tt => ($($rest:tt)*)) => {
        #[test]
        fn $file() {
            let (i, _) = test!(@interpret concat!(stringify!($file), ".ly"));
            test!(@munch i; $($rest)*);
        }
    };

    // Test & assure panic
    ($file:tt => panic) => {
        #[test]
        #[should_panic]
        fn $file() {
            let (_, _) = test!(@interpret concat!(stringify!($file), ".ly"));
        }
    };

    // Test against `stdout`
    ($file:tt => $expected:expr) => {
        #[test]
        fn $file() {
            let (_, out) = test!(@interpret concat!(stringify!($file), ".ly"));
            assert_eq!(out, $expected);
        }
    };
}

#[cfg(test)]
mod builtins;

#[cfg(test)]
mod feature;

#[cfg(test)]
mod implementation;
