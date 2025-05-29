// allow unused b/c the rust lsp fails to recognize imports in submodules

#[allow(unused)]
use crate::{interpreter::*, lexer::*, parser::*};

#[allow(unused)]
macro_rules! var_eq_literal {
    ($interpreter:expr, $id:tt, $token:expr) => {
        assert_eq!(
            *$interpreter.get(&ID::new($id.clone())).unwrap(),
            Variable::Owned(ASTNode::Literal($token)).into(),
        );
    };
}

#[allow(unused)]
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
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn global_variables() {
        let mut i = Interpreter::new();
        let ast = Parser::new(
            Lexer::new()
                .lex(include_str!("feature/global_variables.ly").to_string())
                .unwrap(),
        )
        .parse()
        .unwrap();
        i.execute(ast).unwrap();

        var_eq_literal!(i, "a", Token::Number(1.));
        var_eq_literal!(i, "b", Token::Bool(true));
        var_eq_literal!(i, "c", Token::Str("str".into()));
        var_eq_literal!(i, "d", Token::Char('c'));
    }

    #[test]
    fn math() {
        let mut i = Interpreter::new();
        let ast = Parser::new(
            Lexer::new()
                .lex(include_str!("feature/math.ly").to_string())
                .unwrap(),
        )
        .parse()
        .unwrap();
        i.execute(ast).unwrap();

        var_eq_literal!(i, "a", Token::Number(1.));
        var_eq_literal!(i, "b", Token::Number(2.5));
        var_eq_literal!(i, "c", Token::Number(6.));
    }

    #[test]
    fn conditionals() {
        let mut i = Interpreter::new();
        let ast = Parser::new(
            Lexer::new()
                .lex(include_str!("feature/conditionals.ly").to_string())
                .unwrap(),
        )
        .parse()
        .unwrap();
        i.execute(ast).unwrap();

        var_eq_literal!(i, "a", Token::Number(7.));
    }

    #[test]
    fn functions() {
        let mut i = Interpreter::new();
        let ast = Parser::new(
            Lexer::new()
                .lex(include_str!("feature/functions.ly").to_string())
                .unwrap(),
        )
        .parse()
        .unwrap();
        i.execute(ast).unwrap();

        var_eq_literal!(i, "a", Token::Number(10.));
        var_eq_literal!(i, "b", Token::Number(20.));
        var_eq_literal!(i, "c", Token::Bool(true));
    }

    #[test]
    fn loops() {
        let mut i = Interpreter::new();
        let ast = Parser::new(
            Lexer::new()
                .lex(include_str!("feature/loops.ly").to_string())
                .unwrap(),
        )
        .parse()
        .unwrap();
        i.execute(ast).unwrap();

        var_eq_literal!(i, "i", Token::Number(25.));
        var_eq_literal!(i, "a", Token::Number(25.));
    }

    #[test]
    fn lists() {
        let mut i = Interpreter::new();
        let ast = Parser::new(
            Lexer::new()
                .lex(include_str!("feature/lists.ly").to_string())
                .unwrap(),
        )
        .parse()
        .unwrap();
        i.execute(ast).unwrap();

        var_eq_literal!(i, "idx_a", Token::Number(2.));
        var_eq_literal!(i, "idx_b", Token::Number(3.));
        var_eq_literal!(i, "dangling", Token::Number(10.));
        var_eq!(
            i,
            "idx_list_whole",
            ASTNode::List(SVTable::new_with(vec![ASTNode::Literal(Token::Number(
                123.
            ))
            .into(),]))
        );
        var_eq_literal!(i, "idx_list_part", Token::Number(123.));
        var_eq_literal!(i, "assignment", Token::Number(1.));
    }

    #[test]
    fn imports() {
        let mut i = Interpreter::new();
        let mut p = Parser::new(
            Lexer::new()
                .lex(include_str!("feature/imports.ly").to_string())
                .unwrap(),
        );
        p.set_pwd(PathBuf::from("src/interpreter/tests/feature/"));
        let ast = p.parse().unwrap();
        i.execute(ast).unwrap();

        assert_eq!(
            *i.get(&ID::new("get_res")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(4.))).into()
        );
        var_eq_literal!(i, "get_res", Token::Number(4.));
        var_eq_literal!(i, "assign_res", Token::Str("reassignment value".into()));
        var_eq_literal!(i, "decl_res", Token::Str("declaration value".into()));
    }

    #[test]
    fn nested_imports() {
        let mut i = Interpreter::new();
        let mut p = Parser::new(
            Lexer::new()
                .lex(include_str!("feature/nested_imports.ly").to_string())
                .unwrap(),
        );
        p.set_pwd(PathBuf::from("src/interpreter/tests/feature/"));
        let ast = p.parse().unwrap();
        i.execute(ast).unwrap();

        var_eq_literal!(i, "res", Token::Number(4.));
    }

    #[test]
    fn structs() {
        let mut i = Interpreter::new();
        let ast = Parser::new(
            Lexer::new()
                .lex(include_str!("feature/structs.ly").to_string())
                .unwrap(),
        )
        .parse()
        .unwrap();
        i.execute(ast).unwrap();

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
        let mut i = Interpreter::new();
        let ast = Parser::new(
            Lexer::new()
                .lex(include_str!("implementation/fibonacci.ly").to_string())
                .unwrap(),
        )
        .parse()
        .unwrap();
        i.execute(ast).unwrap();

        var_eq_literal!(i, "result", Token::Number(21.));
    }

    #[test]
    fn factorial() {
        let mut i = Interpreter::new();
        let ast = Parser::new(
            Lexer::new()
                .lex(include_str!("implementation/factorial.ly").to_string())
                .unwrap(),
        )
        .parse()
        .unwrap();
        i.execute(ast).unwrap();

        var_eq_literal!(i, "six_fac", Token::Number(720.));
        var_eq_literal!(i, "one_fac", Token::Number(1.));
        var_eq_literal!(i, "zero_fac", Token::Number(1.));
    }

    #[test]
    fn matrix_rotation() {
        let mut i = Interpreter::new();
        let ast = Parser::new(
            Lexer::new()
                .lex(include_str!("implementation/matrix_rotation.ly").to_string())
                .unwrap(),
        )
        .parse()
        .unwrap();
        i.execute(ast).unwrap();

        var_eq!(
            i,
            "result",
            ASTNode::List(SVTable::new_with(vec![
                ASTNode::List(SVTable::new_with(vec![
                    ASTNode::Literal(Token::Number(1.)).into(),
                    ASTNode::Literal(Token::Number(4.)).into(),
                    ASTNode::Literal(Token::Number(7.)).into(),
                ]))
                .into(),
                ASTNode::List(SVTable::new_with(vec![
                    ASTNode::Literal(Token::Number(2.)).into(),
                    ASTNode::Literal(Token::Number(5.)).into(),
                    ASTNode::Literal(Token::Number(8.)).into(),
                ]))
                .into(),
                ASTNode::List(SVTable::new_with(vec![
                    ASTNode::Literal(Token::Number(3.)).into(),
                    ASTNode::Literal(Token::Number(6.)).into(),
                    ASTNode::Literal(Token::Number(9.)).into(),
                ]))
                .into(),
            ]))
        );
    }
}
