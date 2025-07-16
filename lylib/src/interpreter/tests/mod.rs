// allow unused b/c the rust lsp fails to recognize imports in submodules
#![allow(unused_macros, unused_imports)]

use crate::{
    interpreter::*,
    lexer::{Token::*, *},
    parser::{ASTNode::*, *},
    *,
};

/// Shorthand for comparing a variable with an owned literal.
macro_rules! var_eq_literal {
    ($interpreter:expr, $id:tt, $token:expr) => {
        assert_eq!(
            $interpreter.get(&ID::new(stringify!($id))).unwrap(),
            Variable::Owned(ASTNode::inner_to_owned(&lit!($token))).into(),
        );
    };
}

/// Shorthand for comparing a variable with any owned value.
macro_rules! var_eq {
    ($interpreter:expr, $id:tt, $node:expr) => {
        assert_eq!(
            $interpreter.get(&ID::new(stringify!($id))).unwrap(),
            Variable::Owned($node).into(),
        );
    };
}

/// Shorthand for entire test cases. The name of the function provided is expected to be the name
/// of the test file, given the file extension is omitted.
macro_rules! test {
    (@munch $interpreter:ident; $lhs:tt == $rhs:expr, $($rest:tt)*) => {
        var_eq!($interpreter, $lhs, $rhs);
    };
    (@munch $interpreter:ident; $lhs:tt == $rhs:expr) => {
        var_eq!($interpreter, $lhs, $rhs);
    };
    (@munch $interpreter:ident; $lhs:tt := $rhs:expr, $($rest:tt)*) => {
        var_eq_literal!($interpreter, $lhs, $rhs);
    };
    (@munch $interpreter:ident; $lhs:tt := $rhs:expr) => {
        var_eq_literal!($interpreter, $lhs, $rhs);
    };

    // equality tests
    ($file:tt => ($($rest:tt)*)) => {
        #[test]
        fn $file() {
            let (i, _) = interpret!(concat!(stringify!($file), ".ly"));
            test!(@munch i; $($rest)*);
        }
    };

    // output tets
    ($file:tt => $expected:expr) => {
        #[test]
        fn $file() {
            let (_, out) = interpret!(concat!(stringify!($file), ".ly"));
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
