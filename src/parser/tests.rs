#![cfg(test)]

use super::*;
use crate::lexer::Lexer;
use Token::*;

#[test]
fn decl() {
    assert_eq!(
        Parser::new(Lexer::new().lex("let number = 1; let boolean = true;".into())).parse(),
        Box::from(ASTNode::Block(vec![
            ASTNode::Assign {
                id: "number".into(),
                value: Box::from(ASTNode::Literal(Number(1.))),
            },
            ASTNode::Assign {
                id: "boolean".into(),
                value: Box::from(ASTNode::Literal(Bool(true))),
            }
        ]))
    );
}

#[test]
fn math() {
    assert_eq!(
        Parser::new(Lexer::new().lex("let x = 1 + 2 - 3 * 4 / 5;".into())).parse(),
        Box::from(ASTNode::Block(vec![ASTNode::Assign {
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
        }]))
    );
    assert_eq!(
        Parser::new(Lexer::new().lex("let x = (1 + 1) + ((1 * 1) + 1);".into())).parse(),
        Box::from(ASTNode::Block(vec![ASTNode::Assign {
            id: "x".into(),
            value: Box::from(ASTNode::Op {
                lhs: Box::from(ASTNode::Op {
                    lhs: Box::from(ASTNode::Literal(Token::Number(1.))),
                    op: Token::Add,
                    rhs: Box::from(ASTNode::Literal(Token::Number(1.))),
                }),
                op: Token::Add,
                rhs: Box::from(ASTNode::Op {
                    lhs: Box::from(ASTNode::Op {
                        lhs: Box::from(ASTNode::Literal(Token::Number(1.))),
                        op: Token::Mul,
                        rhs: Box::from(ASTNode::Literal(Token::Number(1.))),
                    }),
                    op: Token::Add,
                    rhs: Box::from(ASTNode::Literal(Token::Number(1.))),
                }),
            }),
        }]))
    );
}

#[test]
fn comparisons() {
    assert_eq!(
        Parser::new(Lexer::new().lex(
            "let a = 100 < 200; let b = 100 <= 200; let c = 200 > 100; let d = 200 >= 100;".into()
        ))
        .parse(),
        Box::from(ASTNode::Block(vec![
            ASTNode::Assign {
                id: "a".into(),
                value: Box::from(ASTNode::Op {
                    lhs: Box::from(ASTNode::Literal(Token::Number(100.))),
                    op: Token::LogicalL,
                    rhs: Box::from(ASTNode::Literal(Token::Number(200.))),
                })
            },
            ASTNode::Assign {
                id: "b".into(),
                value: Box::from(ASTNode::Op {
                    lhs: Box::from(ASTNode::Literal(Token::Number(100.))),
                    op: Token::LogicalLe,
                    rhs: Box::from(ASTNode::Literal(Token::Number(200.))),
                })
            },
            ASTNode::Assign {
                id: "c".into(),
                value: Box::from(ASTNode::Op {
                    lhs: Box::from(ASTNode::Literal(Token::Number(200.))),
                    op: Token::LogicalG,
                    rhs: Box::from(ASTNode::Literal(Token::Number(100.))),
                })
            },
            ASTNode::Assign {
                id: "d".into(),
                value: Box::from(ASTNode::Op {
                    lhs: Box::from(ASTNode::Literal(Token::Number(200.))),
                    op: Token::LogicalGe,
                    rhs: Box::from(ASTNode::Literal(Token::Number(100.))),
                })
            }
        ]))
    );
}

#[test]
fn conditionals() {
    assert_eq!(
        Parser::new(Lexer::new().lex("if 2 > 1 do; a = b; end;".into())).parse(),
        Box::from(ASTNode::Block(vec![ASTNode::Conditional {
            condition: Box::from(ASTNode::Op {
                lhs: Box::from(ASTNode::Literal(Token::Number(2.))),
                op: LogicalG,
                rhs: Box::from(ASTNode::Literal(Token::Number(1.))),
            }),
            body: Box::from(ASTNode::Block(vec![ASTNode::Assign {
                id: "a".into(),
                value: Box::from(ASTNode::Literal(Token::Identifier("b".into())))
            }]))
        }]))
    );
    assert_eq!(
        Parser::new(Lexer::new().lex("if 2 >= 1 + 1 do; let a = b; end;".into())).parse(),
        Box::from(ASTNode::Block(vec![ASTNode::Conditional {
            condition: Box::from(ASTNode::Op {
                lhs: Box::from(ASTNode::Literal(Token::Number(2.))),
                op: LogicalGe,
                rhs: Box::from(ASTNode::Op {
                    lhs: Box::from(ASTNode::Literal(Token::Number(1.))),
                    op: Token::Add,
                    rhs: Box::from(ASTNode::Literal(Token::Number(1.)))
                }),
            }),
            body: Box::from(ASTNode::Block(vec![ASTNode::Assign {
                id: "a".into(),
                value: Box::from(ASTNode::Literal(Token::Identifier("b".into())))
            }]))
        }]))
    );
}

#[test]
fn functions() {
    assert_eq!(
        Parser::new(
            Lexer::new()
                .lex("func fn a b do; let x = a + b; let y = a - b; return x * y; end;".into())
        )
        .parse(),
        Box::from(ASTNode::Block(vec![ASTNode::Function {
            id: "fn".into(),
            arguments: vec!["a".into(), "b".into()],
            body: Box::from(ASTNode::Block(vec![
                ASTNode::Assign {
                    id: "x".into(),
                    value: Box::from(ASTNode::Op {
                        lhs: Box::from(ASTNode::Literal(Token::Identifier("a".into()))),
                        op: Token::Add,
                        rhs: Box::from(ASTNode::Literal(Token::Identifier("b".into()))),
                    })
                },
                ASTNode::Assign {
                    id: "y".into(),
                    value: Box::from(ASTNode::Op {
                        lhs: Box::from(ASTNode::Literal(Token::Identifier("a".into()))),
                        op: Token::Sub,
                        rhs: Box::from(ASTNode::Literal(Token::Identifier("b".into()))),
                    })
                },
                ASTNode::Return(Box::from(ASTNode::Op {
                    lhs: Box::from(ASTNode::Literal(Token::Identifier("x".into()))),
                    op: Token::Mul,
                    rhs: Box::from(ASTNode::Literal(Token::Identifier("y".into()))),
                }))
            ]))
        },]))
    );
    assert_eq!(
        Parser::new(Lexer::new().lex("fn((1 + 2), (3 + 4))".into())).parse(),
        Box::from(ASTNode::Block(vec![ASTNode::FunctionCall {
            id: "fn".into(),
            arguments: vec![
                Box::from(ASTNode::Op {
                    lhs: Box::from(ASTNode::Literal(Token::Number(1.))),
                    op: Token::Add,
                    rhs: Box::from(ASTNode::Literal(Token::Number(2.))),
                }),
                Box::from(ASTNode::Op {
                    lhs: Box::from(ASTNode::Literal(Token::Number(3.))),
                    op: Token::Add,
                    rhs: Box::from(ASTNode::Literal(Token::Number(4.))),
                }),
            ]
        }]))
    );
    assert_eq!(
        Parser::new(Lexer::new().lex("fna(fnb(1), fnc(2))".into())).parse(),
        Box::from(ASTNode::Block(vec![ASTNode::FunctionCall {
            id: "fna".into(),
            arguments: vec![
                Box::from(ASTNode::FunctionCall {
                    id: "fnb".into(),
                    arguments: vec![Box::from(ASTNode::Literal(Token::Number(1.)))]
                }),
                Box::from(ASTNode::FunctionCall {
                    id: "fnc".into(),
                    arguments: vec![Box::from(ASTNode::Literal(Token::Number(2.)))]
                })
            ]
        }]))
    );
    assert_eq!(
        Parser::new(Lexer::new().lex("fn()".into())).parse(),
        Box::from(ASTNode::Block(vec![ASTNode::FunctionCall {
            id: "fn".into(),
            arguments: vec![]
        }]))
    );
}
