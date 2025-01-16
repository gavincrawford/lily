#![cfg(test)]

use super::*;
use crate::lexer::Lexer;

#[test]
fn decl() {
    assert_eq!(
        Parser::new(Lexer::new().lex("let number = 1; let boolean = true;".into())).parse(),
        ASTNode::Block(vec![
            ASTNode::Declare {
                id: "number".into(),
                value: ASTNode::Literal(Token::Number(1.)).into(),
            }
            .into(),
            ASTNode::Declare {
                id: "boolean".into(),
                value: ASTNode::Literal(Token::Bool(true)).into(),
            }
            .into(),
        ])
        .into()
    );
}

#[test]
fn math() {
    assert_eq!(
        Parser::new(Lexer::new().lex("let x = 1 + 2 - 3 * 4 / 5;".into())).parse(),
        ASTNode::Block(vec![ASTNode::Declare {
            id: "x".into(),
            value: ASTNode::Op {
                lhs: ASTNode::Literal(Token::Number(1.)).into(),
                op: Token::Add,
                rhs: ASTNode::Op {
                    lhs: ASTNode::Literal(Token::Number(2.)).into(),
                    op: Token::Sub,
                    rhs: ASTNode::Op {
                        lhs: ASTNode::Literal(Token::Number(3.)).into(),
                        op: Token::Mul,
                        rhs: ASTNode::Op {
                            lhs: ASTNode::Literal(Token::Number(4.)).into(),
                            op: Token::Div,
                            rhs: ASTNode::Literal(Token::Number(5.)).into(),
                        }
                        .into(),
                    }
                    .into(),
                }
                .into(),
            }
            .into(),
        }
        .into()])
        .into()
    );
    assert_eq!(
        Parser::new(Lexer::new().lex("let x = (1 + 1) + ((1 * 1) + 1);".into())).parse(),
        ASTNode::Block(vec![ASTNode::Declare {
            id: "x".into(),
            value: ASTNode::Op {
                lhs: ASTNode::Op {
                    lhs: ASTNode::Literal(Token::Number(1.)).into(),
                    op: Token::Add,
                    rhs: ASTNode::Literal(Token::Number(1.)).into(),
                }
                .into(),
                op: Token::Add,
                rhs: ASTNode::Op {
                    lhs: ASTNode::Op {
                        lhs: ASTNode::Literal(Token::Number(1.)).into(),
                        op: Token::Mul,
                        rhs: ASTNode::Literal(Token::Number(1.)).into(),
                    }
                    .into(),
                    op: Token::Add,
                    rhs: ASTNode::Literal(Token::Number(1.)).into(),
                }
                .into(),
            }
            .into(),
        }
        .into()])
        .into()
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
            ASTNode::Declare {
                id: "a".into(),
                value: ASTNode::Op {
                    lhs: ASTNode::Literal(Token::Number(100.)).into(),
                    op: Token::LogicalL,
                    rhs: ASTNode::Literal(Token::Number(200.)).into(),
                }
                .into(),
            }
            .into(),
            ASTNode::Declare {
                id: "b".into(),
                value: ASTNode::Op {
                    lhs: ASTNode::Literal(Token::Number(100.)).into(),
                    op: Token::LogicalLe,
                    rhs: ASTNode::Literal(Token::Number(200.)).into(),
                }
                .into(),
            }
            .into(),
            ASTNode::Declare {
                id: "c".into(),
                value: ASTNode::Op {
                    lhs: ASTNode::Literal(Token::Number(200.)).into(),
                    op: Token::LogicalG,
                    rhs: ASTNode::Literal(Token::Number(100.)).into(),
                }
                .into(),
            }
            .into(),
            ASTNode::Declare {
                id: "d".into(),
                value: ASTNode::Op {
                    lhs: ASTNode::Literal(Token::Number(200.)).into(),
                    op: Token::LogicalGe,
                    rhs: ASTNode::Literal(Token::Number(100.)).into(),
                }
                .into(),
            }
            .into(),
        ])
        .into()
    );
}

#[test]
fn conditionals() {
    assert_eq!(
        Parser::new(Lexer::new().lex("if 2 > 1 do; a = b; end;".into())).parse(),
        ASTNode::Block(vec![ASTNode::Conditional {
            condition: ASTNode::Op {
                lhs: ASTNode::Literal(Token::Number(2.)).into(),
                op: Token::LogicalG,
                rhs: ASTNode::Literal(Token::Number(1.)).into(),
            }
            .into(),
            if_body: ASTNode::Block(vec![ASTNode::Assign {
                id: "a".into(),
                value: ASTNode::Literal(Token::Identifier("b".into())).into(),
            }
            .into()])
            .into(),
            else_body: ASTNode::Block(vec![]).into(),
        }
        .into()])
        .into()
    );
    assert_eq!(
        Parser::new(Lexer::new().lex("if 1 > 2 do; a = b; else; b = a; end;".into())).parse(),
        ASTNode::Block(vec![ASTNode::Conditional {
            condition: ASTNode::Op {
                lhs: ASTNode::Literal(Token::Number(1.)).into(),
                op: Token::LogicalG,
                rhs: ASTNode::Literal(Token::Number(2.)).into(),
            }
            .into(),
            if_body: ASTNode::Block(vec![ASTNode::Assign {
                id: "a".into(),
                value: ASTNode::Literal(Token::Identifier("b".into())).into(),
            }
            .into()])
            .into(),
            else_body: ASTNode::Block(vec![ASTNode::Assign {
                id: "b".into(),
                value: ASTNode::Literal(Token::Identifier("a".into())).into(),
            }
            .into()])
            .into(),
        }
        .into()])
        .into()
    );
}

#[test]
fn loops() {
    assert_eq!(
        Parser::new(Lexer::new().lex("let i = 0; while i < 10 do; i = i + 1; end;".into())).parse(),
        ASTNode::Block(vec![
            ASTNode::Declare {
                id: "i".into(),
                value: ASTNode::Literal(Token::Number(0.)).into(),
            }
            .into(),
            ASTNode::Loop {
                condition: ASTNode::Op {
                    lhs: ASTNode::Literal(Token::Identifier("i".into())).into(),
                    op: Token::LogicalL,
                    rhs: ASTNode::Literal(Token::Number(10.)).into(),
                }
                .into(),
                body: ASTNode::Block(vec![ASTNode::Assign {
                    id: "i".into(),
                    value: ASTNode::Op {
                        lhs: ASTNode::Literal(Token::Identifier("i".into())).into(),
                        op: Token::Add,
                        rhs: ASTNode::Literal(Token::Number(1.)).into(),
                    }
                    .into()
                }
                .into()])
                .into(),
            }
            .into(),
        ])
        .into()
    )
}

#[test]
fn functions() {
    assert_eq!(
        Parser::new(
            Lexer::new()
                .lex("func fn a b do; let x = a + b; let y = a - b; return x * y; end;".into())
        )
        .parse(),
        ASTNode::Block(vec![ASTNode::Function {
            id: "fn".into(),
            arguments: vec!["a".into(), "b".into()],
            body: ASTNode::Block(vec![
                ASTNode::Declare {
                    id: "x".into(),
                    value: ASTNode::Op {
                        lhs: ASTNode::Literal(Token::Identifier("a".into())).into(),
                        op: Token::Add,
                        rhs: ASTNode::Literal(Token::Identifier("b".into())).into(),
                    }
                    .into(),
                }
                .into(),
                ASTNode::Declare {
                    id: "y".into(),
                    value: ASTNode::Op {
                        lhs: ASTNode::Literal(Token::Identifier("a".into())).into(),
                        op: Token::Sub,
                        rhs: ASTNode::Literal(Token::Identifier("b".into())).into(),
                    }
                    .into(),
                }
                .into(),
                ASTNode::Return(
                    ASTNode::Op {
                        lhs: ASTNode::Literal(Token::Identifier("x".into())).into(),
                        op: Token::Mul,
                        rhs: ASTNode::Literal(Token::Identifier("y".into())).into(),
                    }
                    .into()
                )
                .into(),
            ])
            .into()
        }
        .into(),])
        .into()
    );
    assert_eq!(
        Parser::new(Lexer::new().lex("fn((1 + 2), (3 + 4))".into())).parse(),
        ASTNode::Block(vec![ASTNode::FunctionCall {
            id: "fn".into(),
            arguments: vec![
                ASTNode::Op {
                    lhs: ASTNode::Literal(Token::Number(1.)).into(),
                    op: Token::Add,
                    rhs: ASTNode::Literal(Token::Number(2.)).into(),
                }
                .into(),
                ASTNode::Op {
                    lhs: ASTNode::Literal(Token::Number(3.)).into(),
                    op: Token::Add,
                    rhs: ASTNode::Literal(Token::Number(4.)).into(),
                }
                .into(),
            ]
        }
        .into()])
        .into(),
    );
    assert_eq!(
        Parser::new(Lexer::new().lex("fna(fnb(1), fnc(2))".into())).parse(),
        ASTNode::Block(vec![ASTNode::FunctionCall {
            id: "fna".into(),
            arguments: vec![
                ASTNode::FunctionCall {
                    id: "fnb".into(),
                    arguments: vec![ASTNode::Literal(Token::Number(1.)).into()]
                }
                .into(),
                ASTNode::FunctionCall {
                    id: "fnc".into(),
                    arguments: vec![ASTNode::Literal(Token::Number(2.)).into()]
                }
                .into()
            ]
        }
        .into()])
        .into()
    );
    assert_eq!(
        Parser::new(Lexer::new().lex("fn()".into())).parse(),
        ASTNode::Block(
            vec![ASTNode::FunctionCall {
                id: "fn".into(),
                arguments: vec![]
            }
            .into()]
            .into()
        )
        .into()
    );
}
