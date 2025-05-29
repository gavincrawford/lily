// allow unused b/c the rust lsp fails to recognize imports in submodules
#![allow(unused_macros, unused_imports)]

use crate::{interpreter::*, lexer::*, parser::*};
use astnode::ASTNode::*;
use std::path::PathBuf;

/// Shorthand for comparing a variable with an owned literal.
macro_rules! var_eq_literal {
    ($interpreter:expr, $id:tt, $token:expr) => {
        assert_eq!(
            *$interpreter.get(&ID::new($id.clone())).unwrap(),
            Variable::Owned(Literal($token)).into(),
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

/// Shorthand for executing test code located at the provided path.
macro_rules! interpret {
    ($path:expr) => {{
        let mut i = Interpreter::new();
        let mut p = Parser::new(Lexer::new().lex(include_str!($path).to_string()).unwrap());
        p.set_pwd(PathBuf::from("src/interpreter/tests/feature/"));
        let ast = p.parse().unwrap();
        i.execute(ast).unwrap();
        i
    }};
}

#[cfg(test)]
mod feature {
    use super::*;

    #[test]
    fn global_variables() {
        let i = interpret!("feature/global_variables.ly");
        var_eq_literal!(i, "a", Token::Number(1.));
        var_eq_literal!(i, "b", Token::Bool(true));
        var_eq_literal!(i, "c", Token::Str("str".into()));
        var_eq_literal!(i, "d", Token::Char('c'));
    }

    #[test]
    fn math() {
        let i = interpret!("feature/math.ly");
        var_eq_literal!(i, "a", Token::Number(1.));
        var_eq_literal!(i, "b", Token::Number(2.5));
        var_eq_literal!(i, "c", Token::Number(6.));
    }

    #[test]
    fn conditionals() {
        let i = interpret!("feature/conditionals.ly");
        var_eq_literal!(i, "a", Token::Number(7.));
    }

    #[test]
    fn functions() {
        let i = interpret!("feature/functions.ly");
        var_eq_literal!(i, "a", Token::Number(10.));
        var_eq_literal!(i, "b", Token::Number(20.));
        var_eq_literal!(i, "c", Token::Bool(true));
    }

    #[test]
    fn loops() {
        let i = interpret!("feature/loops.ly");
        var_eq_literal!(i, "i", Token::Number(25.));
        var_eq_literal!(i, "a", Token::Number(25.));
    }

    #[test]
    fn lists() {
        let i = interpret!("feature/lists.ly");
        var_eq_literal!(i, "idx_a", Token::Number(2.));
        var_eq_literal!(i, "idx_b", Token::Number(3.));
        var_eq_literal!(i, "dangling", Token::Number(10.));
        var_eq!(
            i,
            "idx_list_whole",
            List(SVTable::new_with(
                vec![Literal(Token::Number(123.)).into(),]
            ))
        );
        var_eq_literal!(i, "idx_list_part", Token::Number(123.));
        var_eq_literal!(i, "assignment", Token::Number(1.));
    }

    #[test]
    fn imports() {
        let i = interpret!("feature/imports.ly");
        assert_eq!(
            *i.get(&ID::new("get_res")).unwrap(),
            Variable::Owned(Literal(Token::Number(4.))).into()
        );
        var_eq_literal!(i, "get_res", Token::Number(4.));
        var_eq_literal!(i, "assign_res", Token::Str("reassignment value".into()));
        var_eq_literal!(i, "decl_res", Token::Str("declaration value".into()));
    }

    #[test]
    fn nested_imports() {
        let i = interpret!("feature/nested_imports.ly");
        var_eq_literal!(i, "res", Token::Number(4.));
    }

    #[test]
    fn structs() {
        let i = interpret!("feature/structs.ly");
        var_eq_literal!(i, "av", Token::Number(123.));
        var_eq_literal!(i, "bv", Token::Number(0.));
        var_eq_literal!(i, "declaration", Token::Bool(true));
    }
}

#[cfg(test)]
mod implementation {
    use super::*;

    #[test]
    fn fibonacci() {
        let i = interpret!("implementation/fibonacci.ly");
        var_eq_literal!(i, "result", Token::Number(21.));
    }

    #[test]
    fn factorial() {
        let i = interpret!("implementation/factorial.ly");
        var_eq_literal!(i, "six_fac", Token::Number(720.));
        var_eq_literal!(i, "one_fac", Token::Number(1.));
        var_eq_literal!(i, "zero_fac", Token::Number(1.));
    }

    #[test]
    fn matrix_rotation() {
        let i = interpret!("implementation/matrix_rotation.ly");
        var_eq!(
            i,
            "result",
            List(SVTable::new_with(vec![
                List(SVTable::new_with(vec![
                    Literal(Token::Number(1.)).into(),
                    Literal(Token::Number(4.)).into(),
                    Literal(Token::Number(7.)).into(),
                ]))
                .into(),
                List(SVTable::new_with(vec![
                    Literal(Token::Number(2.)).into(),
                    Literal(Token::Number(5.)).into(),
                    Literal(Token::Number(8.)).into(),
                ]))
                .into(),
                List(SVTable::new_with(vec![
                    Literal(Token::Number(3.)).into(),
                    Literal(Token::Number(6.)).into(),
                    Literal(Token::Number(9.)).into(),
                ]))
                .into(),
            ]))
        );
    }
}
