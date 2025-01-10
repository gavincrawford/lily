#![cfg(test)]

use super::*;
use crate::lexer::Lexer;
use Token::*;

#[test]
fn decl() {
    assert_eq!(
        Parser::new(Lexer::new().lex("let x = 1;".into())).parse(),
        ASTNode::Block(vec![ASTNode::Variable {
            id: "x".into(),
            value: Box::from(ASTNode::Literal(Number(1.))),
        }])
    );
}

#[test]
fn math() {
    assert_eq!(
        Parser::new(Lexer::new().lex("let x = 1 + 2 - 3 * 4 / 5;".into())).parse(),
        ASTNode::Block(vec![ASTNode::Variable {
            id: "x".into(),
            value: Box::from(ASTNode::Op {
                lhs: Box::from(ASTNode::Literal(Token::Number(1.))),
                op: Token::Add,
                rhs: Box::from(ASTNode::Op {
                    lhs: Box::from(ASTNode::Literal(Token::Number(2.))),
                    op: Token::Sub,
                    rhs: Box::from(ASTNode::Op {
                        lhs: Box::from(ASTNode::Literal(Token::Number(3.))),
                        op: Token::Mul,
                        rhs: Box::from(ASTNode::Op {
                            lhs: Box::from(ASTNode::Literal(Token::Number(4.))),
                            op: Token::Div,
                            rhs: Box::from(ASTNode::Literal(Token::Number(5.))),
                        }),
                    }),
                }),
            })
        }])
    );
}

#[test]
fn comparisons() {
    assert_eq!(
        Parser::new(Lexer::new().lex(
            "let a = 100 < 200; let b = 100 <= 200; let c = 200 > 100; let d = 200 >= 100;".into()
        ))
        .parse(),
        ASTNode::Block(vec![
            ASTNode::Variable {
                id: "a".into(),
                value: Box::from(ASTNode::Op {
                    lhs: Box::from(ASTNode::Literal(Token::Number(100.))),
                    op: Token::LogicalL,
                    rhs: Box::from(ASTNode::Literal(Token::Number(200.))),
                })
            },
            ASTNode::Variable {
                id: "b".into(),
                value: Box::from(ASTNode::Op {
                    lhs: Box::from(ASTNode::Literal(Token::Number(100.))),
                    op: Token::LogicalLe,
                    rhs: Box::from(ASTNode::Literal(Token::Number(200.))),
                })
            },
            ASTNode::Variable {
                id: "c".into(),
                value: Box::from(ASTNode::Op {
                    lhs: Box::from(ASTNode::Literal(Token::Number(200.))),
                    op: Token::LogicalG,
                    rhs: Box::from(ASTNode::Literal(Token::Number(100.))),
                })
            },
            ASTNode::Variable {
                id: "d".into(),
                value: Box::from(ASTNode::Op {
                    lhs: Box::from(ASTNode::Literal(Token::Number(200.))),
                    op: Token::LogicalGe,
                    rhs: Box::from(ASTNode::Literal(Token::Number(100.))),
                })
            }
        ])
    );
}

#[test]
fn functions() {
    assert_eq!(
        Parser::new(Lexer::new().lex("func fn a b do; let x = a + b; let y = a - b; end;".into()))
            .parse(),
        ASTNode::Block(vec![ASTNode::Function {
            id: "fn".into(),
            arguments: vec!["a".into(), "b".into()],
            body: Box::from(ASTNode::Block(vec![
                ASTNode::Variable {
                    id: "x".into(),
                    value: Box::from(ASTNode::Op {
                        lhs: Box::from(ASTNode::Literal(Token::Identifier("a".into()))),
                        op: Token::Add,
                        rhs: Box::from(ASTNode::Literal(Token::Identifier("b".into()))),
                    })
                },
                ASTNode::Variable {
                    id: "y".into(),
                    value: Box::from(ASTNode::Op {
                        lhs: Box::from(ASTNode::Literal(Token::Identifier("a".into()))),
                        op: Token::Sub,
                        rhs: Box::from(ASTNode::Literal(Token::Identifier("b".into()))),
                    })
                }
            ]))
        }])
    );
}
