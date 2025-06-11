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
            $interpreter.get(&ID::new($id.clone())).unwrap(),
            Variable::Owned(ASTNode::inner_to_owned(&lit!($token))).into(),
        );
    };
}

/// Shorthand for comparing a variable with any owned value.
macro_rules! var_eq {
    ($interpreter:expr, $id:tt, $node:expr) => {
        assert_eq!(
            $interpreter.get(&ID::new($id)).unwrap(),
            Variable::Owned($node).into(),
        );
    };
}

#[cfg(test)]
mod builtins;

#[cfg(test)]
mod feature;

#[cfg(test)]
mod implementation;
