//! The interpreter executes an abstract syntax tree.

mod tests;

use crate::{lexer::Token, parser::ASTNode};
use std::{collections::HashMap, rc::Rc};

pub struct Interpreter<'a> {
    /// Variable storage.
    pub variables: HashMap<String, (usize, ASTNode)>,
    /// Function table.
    pub functions: HashMap<String, (usize, &'a Rc<ASTNode>)>,
    /// Scope level.
    scope: usize,
}
impl<'a> Interpreter<'a> {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            scope: 0,
        }
    }

    /// Executes an AST block, typically the head.
    pub fn execute(&mut self, ast: &'a Rc<ASTNode>) {
        if let ASTNode::Block(statements) = &**ast {
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
        let no_var = &(0, ASTNode::Literal(Token::Undefined));
        let var = self.variables.get(&id).unwrap_or(no_var);
        let var = var.clone();

        // make sure it's a literal before returning
        match (var.0, var.1) {
            (_scope, ASTNode::Literal(t)) => t.clone(),
            _ => {
                panic!("invalid AST node in global scope.");
            }
        }
    }

    /// Sets the value of a variable, copying in the process.
    fn set(&mut self, id: String, scope: usize, value: ASTNode) {
        if let Some((old_scope, _)) = self
            .variables
            .insert(id.clone(), (self.scope, value.clone()))
        {
            self.variables.insert(id, (old_scope, value.clone()));
        } else {
            self.variables.insert(id, (scope, value.clone()));
        }
    }

    /// Drops all out-of-scope variables
    fn drop(&mut self) {
        self.variables
            .retain(|_key, (scope, _variable)| *scope <= self.scope);
    }

    /// Executes an individual expression.
    fn execute_expr(&mut self, statement: &'a Rc<ASTNode>) -> Option<Rc<ASTNode>> {
        match &**statement {
            ASTNode::Assign { id, value } => {
                let resolved_expr = &self.execute_expr(&value).unwrap();
                self.set(id.clone(), self.scope, (&*resolved_expr.clone()).clone());
                None
            }
            ASTNode::Function {
                ref id,
                arguments: ref _arguments,
                body: ref _body,
            } => {
                self.functions.insert(id.clone(), (self.scope, statement));
                None
            }
            ASTNode::FunctionCall {
                id,
                arguments: call_args,
            } => {
                // execute function
                let function = self
                    .functions
                    .get(id)
                    .expect(&*format!("no function named {id} in scope."))
                    .1;
                if let ASTNode::Function {
                    id: _id,
                    arguments: fn_args,
                    body,
                } = &**function
                {
                    // push arguments
                    assert_eq!(call_args.len(), fn_args.len());
                    self.scope += 1;
                    for (idx, arg) in fn_args.iter().enumerate() {
                        let arg_expr = call_args.get(idx).unwrap(); // safety: assertion
                        let resolved_expr = self.execute_expr(arg_expr).unwrap().clone();
                        self.set(arg.clone(), self.scope, (&*resolved_expr.clone()).clone());
                    }

                    // execute body
                    if let ASTNode::Block(expressions) = &**body {
                        for expression in expressions {
                            // if return is found, evaluate it
                            if let ASTNode::Return(value) = &**expression {
                                let return_expr = self.execute_expr(&value).unwrap();
                                self.scope -= 1;
                                self.drop();
                                return Some(return_expr);
                            }

                            // otherwise, process this expression
                            self.execute(&expression);
                        }
                    }

                    // if no return, drop scoped variables anyway
                    self.drop();
                    self.scope -= 1;
                }
                None
            }
            ASTNode::Op { lhs, op, rhs } => {
                if let (Some(a), Some(b)) = (self.execute_expr(&lhs), self.execute_expr(&rhs)) {
                    if let (
                        ASTNode::Literal(Token::Number(a)),
                        ASTNode::Literal(Token::Number(b)),
                    ) = (&*a, &*b)
                    {
                        match op {
                            // math operators
                            Token::Add => {
                                return Some(ASTNode::Literal(Token::Number(a + b)).into())
                            }
                            Token::Sub => {
                                return Some(ASTNode::Literal(Token::Number(a - b)).into())
                            }
                            Token::Mul => {
                                return Some(ASTNode::Literal(Token::Number(a * b)).into())
                            }
                            Token::Div => {
                                return Some(ASTNode::Literal(Token::Number(a / b)).into())
                            }
                            // logical operators
                            Token::LogicalG => {
                                return Some(ASTNode::Literal(Token::Bool(a > b)).into())
                            }
                            Token::LogicalGe => {
                                return Some(ASTNode::Literal(Token::Bool(a >= b)).into())
                            }
                            Token::LogicalL => {
                                return Some(ASTNode::Literal(Token::Bool(a < b)).into())
                            }
                            Token::LogicalLe => {
                                return Some(ASTNode::Literal(Token::Bool(a <= b)).into())
                            }
                            _ => {
                                panic!("operator not implemented.");
                            }
                        }
                    }
                    return None;
                } else {
                    return None;
                }
            }
            ASTNode::Conditional { condition, body } => {
                if let Some(condition) = self.execute_expr(&condition) {
                    if let ASTNode::Literal(Token::Bool(true)) = *condition {
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
                    Some(ASTNode::Literal(self.get(identifier.clone())).into())
                } else {
                    // otherwise, return raw literal
                    Some(statement.clone())
                }
            }
            _ => {
                todo!()
            }
        }
    }
}
