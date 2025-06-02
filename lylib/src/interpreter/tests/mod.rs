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
            *$interpreter.get(&ID::new($id.clone())).unwrap(),
            Variable::Owned(ASTNode::inner_to_owned(&lit!($token))).into(),
        );
    };
}

/// Shorthand for comparing a variable with any owned value.
macro_rules! var_eq {
    ($interpreter:expr, $id:tt, $node:expr) => {
        assert_eq!(
            *$interpreter.get(&ID::new($id)).unwrap(),
            Variable::Owned($node).into(),
        );
    };
}

#[cfg(test)]
mod feature {
    use super::*;

    #[test]
    fn global_variables() {
        let i = interpret!("feature/global_variables.ly");
        var_eq_literal!(i, "a", 1);
        var_eq_literal!(i, "b", true);
        var_eq_literal!(i, "c", "str");
        var_eq_literal!(i, "d", 'c');
    }

    #[test]
    fn math() {
        let i = interpret!("feature/math.ly");
        var_eq_literal!(i, "a", 1);
        var_eq_literal!(i, "b", 2.5);
        var_eq_literal!(i, "c", 6);
    }

    #[test]
    fn operators() {
        let i = interpret!("feature/operators.ly");
        var_eq_literal!(i, "n_eq", true);
        var_eq_literal!(i, "n_neq", true);
        var_eq_literal!(i, "n_add", 2);
        var_eq_literal!(i, "n_sub", 0);
        var_eq_literal!(i, "n_mul", 8);
        var_eq_literal!(i, "n_div", 8);
        var_eq_literal!(i, "b_eq", true);
        var_eq_literal!(i, "b_neq", true);
        var_eq_literal!(i, "s_eq", true);
        var_eq_literal!(i, "s_neq", true);
        var_eq_literal!(i, "s_add_s", "abcd");
        var_eq_literal!(i, "c_eq", true);
        var_eq_literal!(i, "c_neq", true);
    }

    #[test]
    fn conditionals() {
        let i = interpret!("feature/conditionals.ly");
        var_eq_literal!(i, "a", 8);
    }

    #[test]
    fn functions() {
        let i = interpret!("feature/functions.ly");
        var_eq_literal!(i, "a", 10);
        var_eq_literal!(i, "b", 20);
        var_eq_literal!(i, "c", true);
    }

    #[test]
    fn loops() {
        let i = interpret!("feature/loops.ly");
        var_eq_literal!(i, "i", 25);
        var_eq_literal!(i, "a", 25);
    }

    #[test]
    fn lists() {
        let i = interpret!("feature/lists.ly");
        var_eq_literal!(i, "idx_a", 2);
        var_eq_literal!(i, "idx_b", 3);
        var_eq_literal!(i, "dangling", 10);
        var_eq!(i, "idx_list_whole", node!([lit!(123)]));
        var_eq_literal!(i, "idx_list_part", 123);
        var_eq_literal!(i, "assignment", 1);
    }

    #[test]
    fn imports() {
        let i = interpret!("feature/imports.ly");
        var_eq_literal!(i, "get_res", 4);
        var_eq_literal!(i, "assign_res", "reassignment value");
        var_eq_literal!(i, "decl_res", "declaration value");
    }

    #[test]
    fn nested_imports() {
        let i = interpret!("feature/nested_imports.ly");
        var_eq_literal!(i, "res", 4);
    }

    #[test]
    fn structs() {
        let i = interpret!("feature/structs.ly");
        var_eq_literal!(i, "av", 123);
        var_eq_literal!(i, "bv", 0);
        var_eq_literal!(i, "declaration", true);
    }
}

#[cfg(test)]
mod implementation {
    use super::*;

    #[test]
    fn binary_search() {
        let i = interpret!("implementation/binary_search.ly");
        var_eq_literal!(i, "result", 4);
    }

    #[test]
    fn fibonacci() {
        let i = interpret!("implementation/fibonacci.ly");
        var_eq_literal!(i, "result", 21);
    }

    #[test]
    fn factorial() {
        let i = interpret!("implementation/factorial.ly");
        var_eq_literal!(i, "six_fac", 720);
        var_eq_literal!(i, "one_fac", 1);
        var_eq_literal!(i, "zero_fac", 1);
    }

    #[test]
    fn matrix_rotation() {
        let i = interpret!("implementation/matrix_rotation.ly");
        var_eq!(
            i,
            "result",
            node!([
                node!([lit!(1), lit!(4), lit!(7)]),
                node!([lit!(2), lit!(5), lit!(8)]),
                node!([lit!(3), lit!(6), lit!(9)])
            ])
        );
    }
}
