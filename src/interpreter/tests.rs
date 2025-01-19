#[allow(unused)] // the rust lsp fails to recognize imports inside submodules
use crate::{interpreter::*, lexer::*, parser::*};

#[cfg(test)]
mod feature {
    use super::*;

    #[test]
    fn global_variables() {
        let mut i = Interpreter::new();
        let ast =
            Parser::new(Lexer::new().lex(include_str!("tests/global_variables.ly").to_string()))
                .parse();
        i.execute(&ast);

        assert_eq!(
            *i.get("a"),
            Variable::Owned(ASTNode::Literal(Token::Number(1.)))
        );
        assert_eq!(
            *i.get("b"),
            Variable::Owned(ASTNode::Literal(Token::Bool(true)))
        );
        assert_eq!(
            *i.get("c"),
            Variable::Owned(ASTNode::Literal(Token::Str("str".to_string())))
        );
        assert_eq!(
            *i.get("d"),
            Variable::Owned(ASTNode::Literal(Token::Char('c')))
        );
    }

    #[test]
    fn math() {
        let mut i = Interpreter::new();
        let ast = Parser::new(Lexer::new().lex(include_str!("tests/math.ly").to_string())).parse();
        i.execute(&ast);

        assert_eq!(
            *i.get("a"),
            Variable::Owned(ASTNode::Literal(Token::Number(1.)))
        );
        assert_eq!(
            *i.get("b"),
            Variable::Owned(ASTNode::Literal(Token::Number(2.5)))
        );
        assert_eq!(
            *i.get("c"),
            Variable::Owned(ASTNode::Literal(Token::Number(6.)))
        );
    }

    #[test]
    fn conditionals() {
        let mut i = Interpreter::new();
        let ast = Parser::new(Lexer::new().lex(include_str!("tests/conditionals.ly").to_string()))
            .parse();
        i.execute(&ast);

        assert_eq!(
            *i.get("a"),
            Variable::Owned(ASTNode::Literal(Token::Number(5.)))
        );
    }

    #[test]
    fn functions() {
        let mut i = Interpreter::new();
        let ast =
            Parser::new(Lexer::new().lex(include_str!("tests/functions.ly").to_string())).parse();
        i.execute(&ast);

        assert_eq!(
            *i.get("a"),
            Variable::Owned(ASTNode::Literal(Token::Number(10.)))
        );
        assert_eq!(
            *i.get("b"),
            Variable::Owned(ASTNode::Literal(Token::Number(20.)))
        );
        assert_eq!(
            *i.get("c"),
            Variable::Owned(ASTNode::Literal(Token::Bool(true)))
        );
    }

    #[test]
    fn loops() {
        let mut i = Interpreter::new();
        let ast = Parser::new(Lexer::new().lex(include_str!("tests/loops.ly").to_string())).parse();
        i.execute(&ast);

        assert_eq!(
            *i.get("i"),
            Variable::Owned(ASTNode::Literal(Token::Number(25.)))
        );
        assert_eq!(
            *i.get("a"),
            Variable::Owned(ASTNode::Literal(Token::Number(25.)))
        );
    }
}

#[cfg(test)]
mod implementation {
    use super::*;

    #[test]
    fn fibonacci() {
        let mut i = Interpreter::new();
        let ast =
            Parser::new(Lexer::new().lex(include_str!("tests/fibonacci.ly").to_string())).parse();
        i.execute(&ast);

        assert_eq!(
            *i.get("result"),
            Variable::Owned(ASTNode::Literal(Token::Number(21.)))
        );
    }

    #[test]
    fn factorial() {
        let mut i = Interpreter::new();
        let ast =
            Parser::new(Lexer::new().lex(include_str!("tests/factorial.ly").to_string())).parse();
        i.execute(&ast);

        assert_eq!(
            *i.get("six_fac"),
            Variable::Owned(ASTNode::Literal(Token::Number(720.)))
        );
        assert_eq!(
            *i.get("one_fac"),
            Variable::Owned(ASTNode::Literal(Token::Number(1.)))
        );
        assert_eq!(
            *i.get("zero_fac"),
            Variable::Owned(ASTNode::Literal(Token::Number(1.)))
        );
    }
}
