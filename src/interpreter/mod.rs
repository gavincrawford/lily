//! The interpreter executes an abstract syntax tree.

mod tests;

use crate::{lexer::Token, parser::ASTNode};
use std::collections::HashMap;

pub struct Interpreter {
    /// Variable storage.
    pub variables: HashMap<String, (usize, Box<ASTNode>)>,
    /// Scope level.
    scope: usize,
}
impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            scope: 0,
        }
    }

    /// Executes an AST block, typically the head.
    pub fn execute(&mut self, ast: Box<ASTNode>) {
        if let ASTNode::Block(statements) = *ast {
            for statement in statements {
                self.execute_expr(statement);
            }
        } else {
            panic!("AST segment is not a block.");
        }
    }

    /// Gets the value of a variable.
    #[allow(unused)] // only used in tests
    fn get(&self, id: String) -> Token {
        // unwrap variable, or undefined
        let no_var = &(0, Box::from(ASTNode::Literal(Token::Undefined)));
        let var = self.variables.get(&id).unwrap_or(no_var);
        let var = var.clone();

        // make sure it's a literal before returning
        match (var.0, *var.1) {
            (_scope, ASTNode::Literal(t)) => t,
            _ => {
                panic!("invalid AST node in global scope.");
            }
        }
    }

    /// Sets the value of a variable.
    fn set(&mut self, id: String, scope: usize, value: Box<ASTNode>) {
        if let Some((old_scope, _)) = self
            .variables
            .insert(id.clone(), (self.scope, value.clone()))
        {
            self.variables.insert(id, (old_scope, value));
        } else {
            self.variables.insert(id, (scope, value));
        }
    }

    /// Drops all out-of-scope variables
    fn drop(&mut self) {
        self.variables
            .retain(|_key, (scope, _variable)| *scope <= self.scope);
    }

    /// Executes an individual expression.
    fn execute_expr(&mut self, statement: ASTNode) -> Option<ASTNode> {
        match statement {
            ASTNode::Assign { id, value } => {
                let resolved_expr = self.execute_expr(*value).unwrap();
                self.set(id, self.scope, Box::from(resolved_expr));
                None
            }
            ASTNode::Function {
                ref id,
                arguments: ref _arguments,
                body: ref _body,
            } => {
                // TODO can't clone the entire body, too expensive.
                self.set(id.clone(), self.scope, Box::from(statement.clone()));
                None
            }
            ASTNode::FunctionCall {
                id,
                arguments: call_args,
            } => {
                // collect arguments
                let mut res_args = vec![];
                for argument in call_args {
                    res_args.push(self.execute_expr(*argument).unwrap());
                }

                // execute function
                let function = self
                    .variables
                    .get(&id)
                    .expect(&*format!("no function named {id} in scope."));
                // TODO really shouldn't clone here either.
                let function_ast = function.1.clone();
                if let ASTNode::Function {
                    id: _id,
                    arguments: fn_args,
                    body,
                } = *function_ast
                {
                    // push args as variables
                    assert_eq!(fn_args.len(), res_args.len());
                    for (idx, arg) in fn_args.iter().enumerate() {
                        // saftey: assertion
                        self.set(
                            arg.clone(),
                            self.scope,
                            Box::from(res_args.get(idx).unwrap().to_owned()),
                        );
                    }

                    // execute body
                    if let ASTNode::Block(expressions) = *body {
                        for expression in expressions {
                            // if return is found, evaluate it
                            if let ASTNode::Return(value) = expression {
                                return Some(self.execute_expr(*value).unwrap());
                            }

                            // otherwise, process this expression
                            self.execute(Box::from(expression));
                        }
                    }
                }
                None
            }
            ASTNode::Op { lhs, op, rhs } => {
                let a = self.execute_expr(*lhs);
                let b = self.execute_expr(*rhs);
                if let (
                    Some(ASTNode::Literal(Token::Number(a))),
                    Some(ASTNode::Literal(Token::Number(b))),
                ) = (a, b)
                {
                    match op {
                        // math operators
                        Token::Add => return Some(ASTNode::Literal(Token::Number(a + b))),
                        Token::Sub => return Some(ASTNode::Literal(Token::Number(a - b))),
                        Token::Mul => return Some(ASTNode::Literal(Token::Number(a * b))),
                        Token::Div => return Some(ASTNode::Literal(Token::Number(a / b))),
                        // logical operators
                        Token::LogicalG => return Some(ASTNode::Literal(Token::Bool(a > b))),
                        Token::LogicalGe => return Some(ASTNode::Literal(Token::Bool(a >= b))),
                        Token::LogicalL => return Some(ASTNode::Literal(Token::Bool(a < b))),
                        Token::LogicalLe => return Some(ASTNode::Literal(Token::Bool(a <= b))),
                        _ => {
                            panic!("operator not implemented.");
                        }
                    }
                } else {
                    return None;
                }
            }
            ASTNode::Conditional { condition, body } => {
                if let Some(ASTNode::Literal(Token::Bool(cond_true))) =
                    self.execute_expr(*condition)
                {
                    if cond_true {
                        // increase scope level and execute body statements
                        self.scope += 1;
                        self.execute(body);

                        // after finishing, decrease scope level and drop locals
                        self.scope -= 1;
                        self.drop();
                    }
                }
                None
            }
            ASTNode::Literal(ref t) => {
                if let Token::Identifier(identifier) = t {
                    // get variable value if applicable
                    Some(ASTNode::Literal(self.get(identifier.clone())))
                } else {
                    // otherwise, return raw literal
                    Some(statement)
                }
            }
            _ => {
                todo!()
            }
        }
    }
}
