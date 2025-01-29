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
                .lex(include_str!("tests/global_variables.ly").to_string())
                .unwrap(),
        )
        .parse()
        .unwrap();
        i.execute(&ast).unwrap();

        assert_eq!(
            *i.get(&ID::new("a")),
            Variable::Owned(ASTNode::Literal(Token::Number(1.)))
        );
        assert_eq!(
            *i.get(&ID::new("b")),
            Variable::Owned(ASTNode::Literal(Token::Bool(true)))
        );
        assert_eq!(
            *i.get(&ID::new("c")),
            Variable::Owned(ASTNode::Literal(Token::Str("str".to_string())))
        );
        assert_eq!(
            *i.get(&ID::new("d")),
            Variable::Owned(ASTNode::Literal(Token::Char('c')))
        );
    }

    #[test]
    fn math() {
        let mut i = Interpreter::new();
        let ast = Parser::new(
            Lexer::new()
                .lex(include_str!("tests/math.ly").to_string())
                .unwrap(),
        )
        .parse()
        .unwrap();
        i.execute(&ast).unwrap();

        assert_eq!(
            *i.get(&ID::new("a")),
            Variable::Owned(ASTNode::Literal(Token::Number(1.)))
        );
        assert_eq!(
            *i.get(&ID::new("b")),
            Variable::Owned(ASTNode::Literal(Token::Number(2.5)))
        );
        assert_eq!(
            *i.get(&ID::new("c")),
            Variable::Owned(ASTNode::Literal(Token::Number(6.)))
        );
    }

    #[test]
    fn conditionals() {
        let mut i = Interpreter::new();
        let ast = Parser::new(
            Lexer::new()
                .lex(include_str!("tests/conditionals.ly").to_string())
                .unwrap(),
        )
        .parse()
        .unwrap();
        i.execute(&ast).unwrap();

        assert_eq!(
            *i.get(&ID::new("a")),
            Variable::Owned(ASTNode::Literal(Token::Number(5.)))
        );
    }

    #[test]
    fn functions() {
        let mut i = Interpreter::new();
        let ast = Parser::new(
            Lexer::new()
                .lex(include_str!("tests/functions.ly").to_string())
                .unwrap(),
        )
        .parse()
        .unwrap();
        i.execute(&ast).unwrap();

        assert_eq!(
            *i.get(&ID::new("a")),
            Variable::Owned(ASTNode::Literal(Token::Number(10.)))
        );
        assert_eq!(
            *i.get(&ID::new("b")),
            Variable::Owned(ASTNode::Literal(Token::Number(20.)))
        );
        assert_eq!(
            *i.get(&ID::new("c")),
            Variable::Owned(ASTNode::Literal(Token::Bool(true)))
        );
    }

    #[test]
    fn loops() {
        let mut i = Interpreter::new();
        let ast = Parser::new(
            Lexer::new()
                .lex(include_str!("tests/loops.ly").to_string())
                .unwrap(),
        )
        .parse()
        .unwrap();
        i.execute(&ast).unwrap();

        assert_eq!(
            *i.get(&ID::new("i")),
            Variable::Owned(ASTNode::Literal(Token::Number(25.)))
        );
        assert_eq!(
            *i.get(&ID::new("a")),
            Variable::Owned(ASTNode::Literal(Token::Number(25.)))
        );
    }

    #[test]
    fn lists() {
        let mut i = Interpreter::new();
        let ast = Parser::new(
            Lexer::new()
                .lex(include_str!("tests/lists.ly").to_string())
                .unwrap(),
        )
        .parse()
        .unwrap();
        i.execute(&ast).unwrap();

        assert_eq!(
            *i.get(&ID::new("idx_a")),
            Variable::Owned(ASTNode::Literal(Token::Number(1.)))
        );
        assert_eq!(
            *i.get(&ID::new("idx_b")),
            Variable::Owned(ASTNode::Literal(Token::Number(2.)))
        );
        assert_eq!(
            *i.get(&ID::new("idx_c")),
            Variable::Owned(ASTNode::Literal(Token::Number(3.)))
        );
        assert_eq!(
            *i.get(&ID::new("idx_d")),
            Variable::Owned(ASTNode::Literal(Token::Number(4.)))
        );
    }

    #[test]
    fn imports() {
        let mut i = Interpreter::new();
        let mut p = Parser::new(
            Lexer::new()
                .lex(include_str!("tests/imports.ly").to_string())
                .unwrap(),
        );
        p.set_pwd(PathBuf::from("src/interpreter/tests/"));
        let ast = p.parse().unwrap();
        i.execute(&ast).unwrap();

        assert_eq!(
            *i.get(&ID::new("res")),
            Variable::Owned(ASTNode::Literal(Token::Number(4.)))
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
                .lex(include_str!("tests/fibonacci.ly").to_string())
                .unwrap(),
        )
        .parse()
        .unwrap();
        i.execute(&ast).unwrap();

        assert_eq!(
            *i.get(&ID::new("result")),
            Variable::Owned(ASTNode::Literal(Token::Number(21.)))
        );
    }

    #[test]
    fn factorial() {
        let mut i = Interpreter::new();
        let ast = Parser::new(
            Lexer::new()
                .lex(include_str!("tests/factorial.ly").to_string())
                .unwrap(),
        )
        .parse()
        .unwrap();
        i.execute(&ast).unwrap();

        assert_eq!(
            *i.get(&ID::new("six_fac")),
            Variable::Owned(ASTNode::Literal(Token::Number(720.)))
        );
        assert_eq!(
            *i.get(&ID::new("one_fac")),
            Variable::Owned(ASTNode::Literal(Token::Number(1.)))
        );
        assert_eq!(
            *i.get(&ID::new("zero_fac")),
            Variable::Owned(ASTNode::Literal(Token::Number(1.)))
        );
    }
}
