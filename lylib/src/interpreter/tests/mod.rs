#[allow(unused)] // the rust lsp fails to recognize imports inside submodules
use crate::{interpreter::*, lexer::*, parser::*};

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

        assert_eq!(
            *i.get(&ID::new("a")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(1.))).into()
        );
        assert_eq!(
            *i.get(&ID::new("b")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Bool(true))).into()
        );
        assert_eq!(
            *i.get(&ID::new("c")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Str("str".to_string()))).into()
        );
        assert_eq!(
            *i.get(&ID::new("d")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Char('c'))).into()
        );
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

        assert_eq!(
            *i.get(&ID::new("a")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(1.))).into()
        );
        assert_eq!(
            *i.get(&ID::new("b")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(2.5))).into()
        );
        assert_eq!(
            *i.get(&ID::new("c")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(6.))).into()
        );
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

        assert_eq!(
            *i.get(&ID::new("a")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(7.))).into()
        );
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

        assert_eq!(
            *i.get(&ID::new("a")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(10.))).into()
        );
        assert_eq!(
            *i.get(&ID::new("b")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(20.))).into()
        );
        assert_eq!(
            *i.get(&ID::new("c")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Bool(true))).into()
        );
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

        assert_eq!(
            *i.get(&ID::new("i")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(25.))).into()
        );
        assert_eq!(
            *i.get(&ID::new("a")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(25.))).into()
        );
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

        assert_eq!(
            *i.get(&ID::new("idx_a")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(10.))).into()
        );
        assert_eq!(
            *i.get(&ID::new("idx_b")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(2.))).into()
        );
        assert_eq!(
            *i.get(&ID::new("idx_c")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(3.))).into()
        );
        assert_eq!(
            *i.get(&ID::new("idx_list")).unwrap(),
            Variable::Owned(
                ASTNode::List(vec![
                    ASTNode::Literal(Token::Char('a')).into(),
                    ASTNode::Literal(Token::Char('b')).into(),
                    ASTNode::Literal(Token::Char('c')).into(),
                ])
                .into()
            )
            .into()
        );
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
        assert_eq!(
            *i.get(&ID::new("assign_res")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Str("reassignment value".into()))).into()
        );
        assert_eq!(
            *i.get(&ID::new("decl_res")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Str("declaration value".into()))).into()
        );
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

        assert_eq!(
            *i.get(&ID::new("res")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(4.))).into()
        );
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

        assert_eq!(
            *i.get(&ID::new("av")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(123.))).into()
        );
        assert_eq!(
            *i.get(&ID::new("bv")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(0.))).into()
        );
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

        assert_eq!(
            *i.get(&ID::new("result")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(21.))).into()
        );
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

        assert_eq!(
            *i.get(&ID::new("six_fac")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(720.))).into()
        );
        assert_eq!(
            *i.get(&ID::new("one_fac")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(1.))).into()
        );
        assert_eq!(
            *i.get(&ID::new("zero_fac")).unwrap(),
            Variable::Owned(ASTNode::Literal(Token::Number(1.))).into()
        );
    }
}
