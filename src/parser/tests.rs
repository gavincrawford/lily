#![cfg(test)]

use super::*;
use Token::*;

#[test]
fn decl() {
    assert_eq!(
        Parser::new(vec![Let, Identifier("x".into()), Equal, Number(100.), Endl]).parse(),
        ASTNode::Program(vec![ASTNode::Variable {
            id: "x".into(),
            value: Box::from(ASTNode::Literal(Number(100.)))
        }])
    );
    assert_eq!(
        Parser::new(vec![
            Let,
            Identifier("x".into()),
            Equal,
            Identifier("y".into()),
            Endl
        ])
        .parse(),
        ASTNode::Program(vec![ASTNode::Variable {
            id: "x".into(),
            value: Box::from(ASTNode::Literal(Identifier("y".into())))
        }])
    );
    assert_eq!(
        Parser::new(vec![
            Let,
            Identifier("x".into()),
            Equal,
            Number(100.),
            Add,
            Number(100.),
            Endl
        ])
        .parse(),
        ASTNode::Program(vec![ASTNode::Variable {
            id: "x".into(),
            value: Box::from(ASTNode::Op {
                lhs: Box::from(ASTNode::Literal(Token::Number(100.))),
                op: Token::Add,
                rhs: Box::from(ASTNode::Literal(Token::Number(100.))),
            })
        }])
    );
}
