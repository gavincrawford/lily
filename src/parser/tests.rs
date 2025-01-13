#![cfg(test)]

use super::*;
use crate::lexer::Lexer;
use Token::*;

// TODO replace any left over `Rc::from` with `.into()`

#[test]
fn decl() {
    assert_eq!(
        Parser::new(Lexer::new().lex("let number = 1; let boolean = true;".into())).parse(),
        Rc::from(ASTNode::Block(vec![
            ASTNode::Assign {
                id: "number".into(),
                value: Rc::from(ASTNode::Literal(Number(1.))),
            }
            .into(),
            ASTNode::Assign {
                id: "boolean".into(),
                value: Rc::from(ASTNode::Literal(Bool(true))),
            }
            .into(),
        ]))
    );
}

#[test]
fn math() {
    assert_eq!(
        Parser::new(Lexer::new().lex("let x = 1 + 2 - 3 * 4 / 5;".into())).parse(),
        Rc::from(ASTNode::Block(vec![ASTNode::Assign {
            id: "x".into(),
            value: Rc::from(ASTNode::Op {
                lhs: Rc::from(ASTNode::Literal(Token::Number(1.))),
                op: Token::Add,
                rhs: Rc::from(ASTNode::Op {
                    lhs: Rc::from(ASTNode::Literal(Token::Number(2.))),
                    op: Token::Sub,
                    rhs: Rc::from(ASTNode::Op {
                        lhs: Rc::from(ASTNode::Literal(Token::Number(3.))),
                        op: Token::Mul,
                        rhs: Rc::from(ASTNode::Op {
                            lhs: Rc::from(ASTNode::Literal(Token::Number(4.))),
                            op: Token::Div,
                            rhs: Rc::from(ASTNode::Literal(Token::Number(5.))),
                        }),
                    }),
                }),
            })
        }
        .into()]))
    );
    assert_eq!(
        Parser::new(Lexer::new().lex("let x = (1 + 1) + ((1 * 1) + 1);".into())).parse(),
        Rc::from(ASTNode::Block(vec![ASTNode::Assign {
            id: "x".into(),
            value: Rc::from(ASTNode::Op {
                lhs: Rc::from(ASTNode::Op {
                    lhs: Rc::from(ASTNode::Literal(Token::Number(1.))),
                    op: Token::Add,
                    rhs: Rc::from(ASTNode::Literal(Token::Number(1.))),
                }),
                op: Token::Add,
                rhs: Rc::from(ASTNode::Op {
                    lhs: Rc::from(ASTNode::Op {
                        lhs: Rc::from(ASTNode::Literal(Token::Number(1.))),
                        op: Token::Mul,
                        rhs: Rc::from(ASTNode::Literal(Token::Number(1.))),
                    }),
                    op: Token::Add,
                    rhs: Rc::from(ASTNode::Literal(Token::Number(1.))),
                }),
            }),
        }
        .into()]))
    );
}

#[test]
fn comparisons() {
    assert_eq!(
        Parser::new(Lexer::new().lex(
            "let a = 100 < 200; let b = 100 <= 200; let c = 200 > 100; let d = 200 >= 100;".into()
        ))
        .parse(),
        Rc::from(ASTNode::Block(vec![
            ASTNode::Assign {
                id: "a".into(),
                value: Rc::from(ASTNode::Op {
                    lhs: Rc::from(ASTNode::Literal(Token::Number(100.))),
                    op: Token::LogicalL,
                    rhs: Rc::from(ASTNode::Literal(Token::Number(200.))),
                })
            }
            .into(),
            ASTNode::Assign {
                id: "b".into(),
                value: Rc::from(ASTNode::Op {
                    lhs: Rc::from(ASTNode::Literal(Token::Number(100.))),
                    op: Token::LogicalLe,
                    rhs: Rc::from(ASTNode::Literal(Token::Number(200.))),
                })
            }
            .into(),
            ASTNode::Assign {
                id: "c".into(),
                value: Rc::from(ASTNode::Op {
                    lhs: Rc::from(ASTNode::Literal(Token::Number(200.))),
                    op: Token::LogicalG,
                    rhs: Rc::from(ASTNode::Literal(Token::Number(100.))),
                })
            }
            .into(),
            ASTNode::Assign {
                id: "d".into(),
                value: Rc::from(ASTNode::Op {
                    lhs: Rc::from(ASTNode::Literal(Token::Number(200.))),
                    op: Token::LogicalGe,
                    rhs: Rc::from(ASTNode::Literal(Token::Number(100.))),
                })
            }
            .into(),
        ]))
    );
}

#[test]
fn conditionals() {
    assert_eq!(
        Parser::new(Lexer::new().lex("if 2 > 1 do; a = b; end;".into())).parse(),
        Rc::from(ASTNode::Block(vec![ASTNode::Conditional {
            condition: Rc::from(ASTNode::Op {
                lhs: Rc::from(ASTNode::Literal(Token::Number(2.))),
                op: LogicalG,
                rhs: Rc::from(ASTNode::Literal(Token::Number(1.))),
            }),
            body: Rc::from(ASTNode::Block(vec![ASTNode::Assign {
                id: "a".into(),
                value: Rc::from(ASTNode::Literal(Token::Identifier("b".into())))
            }
            .into()]))
        }
        .into()]))
    );
    assert_eq!(
        Parser::new(Lexer::new().lex("if 2 >= 1 + 1 do; let a = b; end;".into())).parse(),
        Rc::from(ASTNode::Block(vec![ASTNode::Conditional {
            condition: Rc::from(ASTNode::Op {
                lhs: Rc::from(ASTNode::Literal(Token::Number(2.))),
                op: LogicalGe,
                rhs: Rc::from(ASTNode::Op {
                    lhs: Rc::from(ASTNode::Literal(Token::Number(1.))),
                    op: Token::Add,
                    rhs: Rc::from(ASTNode::Literal(Token::Number(1.)))
                }),
            }),
            body: Rc::from(ASTNode::Block(vec![ASTNode::Assign {
                id: "a".into(),
                value: Rc::from(ASTNode::Literal(Token::Identifier("b".into())))
            }
            .into()]))
        }
        .into()]))
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
        Rc::from(ASTNode::Block(vec![ASTNode::Function {
            id: "fn".into(),
            arguments: vec!["a".into(), "b".into()],
            body: Rc::from(ASTNode::Block(vec![
                ASTNode::Assign {
                    id: "x".into(),
                    value: Rc::from(ASTNode::Op {
                        lhs: Rc::from(ASTNode::Literal(Token::Identifier("a".into()))),
                        op: Token::Add,
                        rhs: Rc::from(ASTNode::Literal(Token::Identifier("b".into()))),
                    })
                }
                .into(),
                ASTNode::Assign {
                    id: "y".into(),
                    value: Rc::from(ASTNode::Op {
                        lhs: Rc::from(ASTNode::Literal(Token::Identifier("a".into()))),
                        op: Token::Sub,
                        rhs: Rc::from(ASTNode::Literal(Token::Identifier("b".into()))),
                    })
                }
                .into(),
                ASTNode::Return(
                    Rc::from(ASTNode::Op {
                        lhs: Rc::from(ASTNode::Literal(Token::Identifier("x".into()))),
                        op: Token::Mul,
                        rhs: Rc::from(ASTNode::Literal(Token::Identifier("y".into()))),
                    })
                    .into()
                )
                .into()
            ]))
        }
        .into(),]))
    );
    assert_eq!(
        Parser::new(Lexer::new().lex("fn((1 + 2), (3 + 4))".into())).parse(),
        Rc::from(ASTNode::Block(vec![ASTNode::FunctionCall {
            id: "fn".into(),
            arguments: vec![
                Rc::from(ASTNode::Op {
                    lhs: Rc::from(ASTNode::Literal(Token::Number(1.))),
                    op: Token::Add,
                    rhs: Rc::from(ASTNode::Literal(Token::Number(2.))),
                }),
                Rc::from(ASTNode::Op {
                    lhs: Rc::from(ASTNode::Literal(Token::Number(3.))),
                    op: Token::Add,
                    rhs: Rc::from(ASTNode::Literal(Token::Number(4.))),
                }),
            ]
        }
        .into()]))
    );
    assert_eq!(
        Parser::new(Lexer::new().lex("fna(fnb(1), fnc(2))".into())).parse(),
        Rc::from(ASTNode::Block(vec![ASTNode::FunctionCall {
            id: "fna".into(),
            arguments: vec![
                Rc::from(ASTNode::FunctionCall {
                    id: "fnb".into(),
                    arguments: vec![Rc::from(ASTNode::Literal(Token::Number(1.)))]
                }),
                Rc::from(ASTNode::FunctionCall {
                    id: "fnc".into(),
                    arguments: vec![Rc::from(ASTNode::Literal(Token::Number(2.)))]
                })
            ]
        }
        .into()]))
    );
    assert_eq!(
        Parser::new(Lexer::new().lex("fn()".into())).parse(),
        Rc::from(ASTNode::Block(vec![ASTNode::FunctionCall {
            id: "fn".into(),
            arguments: vec![]
        }
        .into()]))
    );
}
