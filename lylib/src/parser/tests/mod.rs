#![cfg(test)]

use super::*;
use crate::lexer::Lexer;
use std::path::PathBuf;

#[test]
fn decl() {
    let result = Parser::new(
        Lexer::new()
            .lex("let number = -1; let boolean = true;".into())
            .unwrap(),
    )
    .parse();
    assert!(result.is_ok(), "Parser failed: {:?}", result);
    assert_eq!(
        result.unwrap(),
        ASTNode::Block(vec![
            ASTNode::Declare {
                target: ASTNode::Literal(Token::Identifier("number".into())).into(),
                value: ASTNode::Literal(Token::Number(-1.)).into(),
            }
            .into(),
            ASTNode::Declare {
                target: ASTNode::Literal(Token::Identifier("boolean".into())).into(),
                value: ASTNode::Literal(Token::Bool(true)).into(),
            }
            .into(),
        ])
        .into()
    );
}

#[test]
fn lists() {
    let result = Parser::new(
        Lexer::new()
            .lex("let list = [1, 2, 3]; let value = list[0];".into())
            .unwrap(),
    )
    .parse();
    assert!(result.is_ok(), "Parser failed: {:?}", result);
    assert_eq!(
        result.unwrap(),
        ASTNode::Block(vec![
            ASTNode::Declare {
                target: ASTNode::Literal(Token::Identifier("list".into())).into(),
                value: ASTNode::List(SVTable::new_with(vec![
                    ASTNode::Literal(Token::Number(1.)).into(),
                    ASTNode::Literal(Token::Number(2.)).into(),
                    ASTNode::Literal(Token::Number(3.)).into(),
                ]))
                .into(),
            }
            .into(),
            ASTNode::Declare {
                target: ASTNode::Literal(Token::Identifier("value".into())).into(),
                value: ASTNode::Index {
                    target: ASTNode::Literal(Token::Identifier("list".into())).into(),
                    index: ASTNode::Literal(Token::Number(0.)).into()
                }
                .into(),
            }
            .into(),
        ])
        .into()
    );
}

#[test]
fn math() {
    let result = Parser::new(
        Lexer::new()
            .lex("let x = 1 + 2 - 3 * 4 / 5;".into())
            .unwrap(),
    )
    .parse();
    assert!(result.is_ok(), "Parser failed: {:?}", result);
    assert_eq!(
        result.unwrap(),
        ASTNode::Block(vec![ASTNode::Declare {
            target: ASTNode::Literal(Token::Identifier("x".into())).into(),
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
}

#[test]
fn comparisons() {
    let result = Parser::new(
        Lexer::new()
            .lex(
                "let a = 100 < 200; let b = 100 <= 200; let c = 200 > 100; let d = 200 >= 100;"
                    .into(),
            )
            .unwrap(),
    )
    .parse();
    assert!(result.is_ok(), "Parser failed: {:?}", result);
    assert_eq!(
        result.unwrap(),
        ASTNode::Block(vec![
            ASTNode::Declare {
                target: ASTNode::Literal(Token::Identifier("a".into())).into(),
                value: ASTNode::Op {
                    lhs: ASTNode::Literal(Token::Number(100.)).into(),
                    op: Token::LogicalL,
                    rhs: ASTNode::Literal(Token::Number(200.)).into(),
                }
                .into(),
            }
            .into(),
            ASTNode::Declare {
                target: ASTNode::Literal(Token::Identifier("b".into())).into(),
                value: ASTNode::Op {
                    lhs: ASTNode::Literal(Token::Number(100.)).into(),
                    op: Token::LogicalLe,
                    rhs: ASTNode::Literal(Token::Number(200.)).into(),
                }
                .into(),
            }
            .into(),
            ASTNode::Declare {
                target: ASTNode::Literal(Token::Identifier("c".into())).into(),
                value: ASTNode::Op {
                    lhs: ASTNode::Literal(Token::Number(200.)).into(),
                    op: Token::LogicalG,
                    rhs: ASTNode::Literal(Token::Number(100.)).into(),
                }
                .into(),
            }
            .into(),
            ASTNode::Declare {
                target: ASTNode::Literal(Token::Identifier("d".into())).into(),
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
    let result = Parser::new(Lexer::new().lex("if 2 > 1 do; a = b; end;".into()).unwrap()).parse();
    assert!(result.is_ok(), "Parser failed: {:?}", result);
    assert_eq!(
        result.unwrap(),
        ASTNode::Block(vec![ASTNode::Conditional {
            condition: ASTNode::Op {
                lhs: ASTNode::Literal(Token::Number(2.)).into(),
                op: Token::LogicalG,
                rhs: ASTNode::Literal(Token::Number(1.)).into(),
            }
            .into(),
            if_body: ASTNode::Block(vec![ASTNode::Assign {
                target: ASTNode::Literal(Token::Identifier("a".into())).into(),
                value: ASTNode::Literal(Token::Identifier("b".into())).into(),
            }
            .into()])
            .into(),
            else_body: ASTNode::Block(vec![]).into(),
        }
        .into()])
        .into()
    );
}

#[test]
fn arguments() {
    let result = Parser::new(
        Lexer::new()
            .lex("let result = func((1 + 1) * 2)".into())
            .unwrap(),
    )
    .parse();
    assert!(result.is_ok(), "Parser failed: {:?}", result);
    assert_eq!(
        result.unwrap(),
        ASTNode::Block(vec![ASTNode::Declare {
            target: ASTNode::Literal(Token::Identifier("result".into())).into(),
            value: ASTNode::FunctionCall {
                target: ASTNode::Literal(Token::Identifier("func".into())).into(),
                arguments: vec![ASTNode::Op {
                    lhs: ASTNode::Op {
                        lhs: ASTNode::Literal(Token::Number(1.)).into(),
                        op: Token::Add,
                        rhs: ASTNode::Literal(Token::Number(1.)).into(),
                    }
                    .into(),
                    op: Token::Mul,
                    rhs: ASTNode::Literal(Token::Number(2.)).into()
                }
                .into()],
            }
            .into()
        }
        .into()])
        .into()
    );
}

#[test]
fn functions() {
    let result = Parser::new(
        Lexer::new()
            .lex("func fn a b do; let x = a + b; let y = a - b; return x * y; end;".into())
            .unwrap(),
    )
    .parse();
    assert!(result.is_ok(), "Parser failed: {:?}", result);
    assert_eq!(
        result.unwrap(),
        ASTNode::Block(vec![ASTNode::Function {
            id: "fn".into(),
            arguments: vec!["a".into(), "b".into()],
            body: ASTNode::Block(vec![
                ASTNode::Declare {
                    target: ASTNode::Literal(Token::Identifier("x".into())).into(),
                    value: ASTNode::Op {
                        lhs: ASTNode::Literal(Token::Identifier("a".into())).into(),
                        op: Token::Add,
                        rhs: ASTNode::Literal(Token::Identifier("b".into())).into(),
                    }
                    .into(),
                }
                .into(),
                ASTNode::Declare {
                    target: ASTNode::Literal(Token::Identifier("y".into())).into(),
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
        .into()])
        .into()
    );
}

#[test]
fn import() {
    let mut parser = Parser::new(
        Lexer::new()
            .lex("import \"./module1.ly\" as mod1; let ten = mod1.mod2.add2(5, 5);".into())
            .unwrap(),
    );
    parser.set_pwd(PathBuf::from("src/parser/tests"));
    assert_eq!(
        parser.parse().unwrap(),
        ASTNode::Block(vec![
            ASTNode::Module {
                alias: Some("mod1".into()),
                body: ASTNode::Block(vec![
                    ASTNode::Module {
                        alias: Some("mod2".into()),
                        body: ASTNode::Block(vec![ASTNode::Function {
                            id: "add2".into(),
                            arguments: vec!["a".into(), "b".into()],
                            body: ASTNode::Block(vec![ASTNode::Return(
                                ASTNode::Op {
                                    lhs: ASTNode::Literal(Token::Identifier("a".into())).into(),
                                    op: Token::Add,
                                    rhs: ASTNode::Literal(Token::Identifier("b".into())).into(),
                                }
                                .into()
                            )
                            .into(),])
                            .into()
                        }
                        .into()])
                        .into()
                    }
                    .into(),
                    ASTNode::Function {
                        id: "add1".into(),
                        arguments: vec!["a".into(), "b".into()],
                        body: ASTNode::Block(vec![ASTNode::Return(
                            ASTNode::Op {
                                lhs: ASTNode::Literal(Token::Identifier("a".into())).into(),
                                op: Token::Add,
                                rhs: ASTNode::Literal(Token::Identifier("b".into())).into(),
                            }
                            .into()
                        )
                        .into(),])
                        .into()
                    }
                    .into(),
                ])
                .into()
            }
            .into(),
            ASTNode::Declare {
                target: ASTNode::Literal(Token::Identifier("ten".into())).into(),
                value: ASTNode::FunctionCall {
                    target: ASTNode::Literal(Token::Identifier("mod1.mod2.add2".into())).into(),
                    arguments: vec![
                        ASTNode::Literal(Token::Number(5.)).into(),
                        ASTNode::Literal(Token::Number(5.)).into()
                    ]
                }
                .into()
            }
            .into()
        ])
        .into()
    );
}

#[test]
fn structs() {
    let mut parser = Parser::new(
        Lexer::new()
            .lex("struct Number do; let value = 0; end; let instance = new Number();".into())
            .unwrap(),
    );
    assert_eq!(
        parser.parse().unwrap(),
        ASTNode::Block(vec![
            ASTNode::Struct {
                id: "Number".into(),
                body: ASTNode::Block(vec![ASTNode::Declare {
                    target: ASTNode::Literal(Token::Identifier("value".into())).into(),
                    value: ASTNode::Literal(Token::Number(0.)).into()
                }
                .into()])
                .into()
            }
            .into(),
            ASTNode::Declare {
                target: ASTNode::Literal(Token::Identifier("instance".into())).into(),
                value: ASTNode::FunctionCall {
                    target: ASTNode::Literal(Token::Identifier("Number".into())).into(),
                    arguments: vec![]
                }
                .into()
            }
            .into()
        ])
        .into()
    );
}
