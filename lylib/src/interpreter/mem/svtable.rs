//! Implements the SVTable, or the scoped-variable table.

use super::{ASTNode, Token, Variable};
use anyhow::{bail, Result};
use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc, slice::Iter};

#[derive(Debug, PartialEq)]
pub struct SVTable {
    /// Holds all the scope frames, each of which hold their respective variables.
    table: Vec<HashMap<String, Rc<RefCell<Variable>>>>,
    /// Holds all the modules defined at this SVTable's scope.
    modules: HashMap<String, Rc<RefCell<SVTable>>>,
}

impl SVTable {
    /// Creates a new scoped-variable table with a default scope.
    pub fn new() -> Self {
        let mut svt = Self {
            table: vec![],
            modules: HashMap::new(),
        };
        svt.add_scope();
        svt
    }

    /// Creates a new scoped-variable table and adds the specified values.
    /// Used for creating list tables.
    pub fn new_with(values: Vec<Rc<ASTNode>>) -> Rc<RefCell<SVTable>> {
        let mut svt = Self {
            table: vec![],
            modules: HashMap::new(),
        };
        svt.add_scope();
        let scope = svt.get_scope(0).unwrap(); // safety ^
        for (idx, value) in values.iter().enumerate() {
            scope.insert(
                idx.to_string(),
                Rc::new(RefCell::new(Variable::Owned(ASTNode::inner_to_owned(
                    value,
                )))),
            );
        }
        Rc::new(RefCell::new(svt))
    }

    /// Returns the iterator to the internal list of frames.
    pub fn iter(&self) -> Iter<'_, HashMap<String, Rc<RefCell<Variable>>>> {
        self.table.iter()
    }

    /// Returns the inner list of frames.
    pub fn inner(&self) -> &Vec<HashMap<String, Rc<RefCell<Variable>>>> {
        &self.table
    }

    /// Returns the inner list of frames, mutable.
    pub fn inner_mut(&mut self) -> &mut Vec<HashMap<String, Rc<RefCell<Variable>>>> {
        &mut self.table
    }

    /// Adds a new module. Returns a reference to the newly created module.
    pub fn add_module(&mut self, name: impl Into<String>) -> Rc<RefCell<SVTable>> {
        let name = name.into();
        self.modules
            .insert(name.to_owned(), RefCell::new(SVTable::new()).into());
        self.modules.get(&name.to_owned()).unwrap().to_owned()
    }

    /// Gets a module by name. Returns an immutable reference to the module if found.
    pub fn get_module(&self, name: impl Into<String>) -> Result<Rc<RefCell<SVTable>>> {
        let name = name.into();
        if let Some(module) = self.modules.get(&name) {
            return Ok(module.clone());
        } else {
            bail!("failed to find module '{}'", name);
        }
    }

    /// Adds a new scope.
    pub fn add_scope(&mut self) {
        self.table.push(HashMap::new());
    }

    /// Gets a scope map. Mutable by default.
    pub fn get_scope(
        &mut self,
        index: usize,
    ) -> Option<&mut HashMap<String, Rc<RefCell<Variable>>>> {
        self.table.get_mut(index)
    }

    /// Returns the number of scopes in this table.
    pub fn scopes(&self) -> usize {
        self.table.len()
    }
}

impl Display for SVTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn prettify(node: Rc<ASTNode>) -> String {
            String::from(match &*node {
                ASTNode::Literal(Token::Identifier(id)) => format!("{}", id),
                ASTNode::Literal(token) => format!("{:?}", token),
                ASTNode::List(list) => format!("[{}]", list.borrow()),
                ASTNode::Op { lhs, op, rhs } => format!(
                    "{} {:?} {}",
                    prettify(lhs.clone()),
                    op,
                    prettify(rhs.clone())
                ),
                ASTNode::Block(lines) => {
                    format!(
                        "{}",
                        lines
                            .iter()
                            .map(|ln| { prettify(ln.clone()) })
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                }
                ASTNode::Return(value) => prettify(value.clone()),
                ASTNode::Function {
                    id,
                    arguments,
                    body,
                } => format!(
                    "{}({}) => {}",
                    id.to_path().join("."),
                    arguments.join(", "),
                    prettify(body.clone())
                ),
                other => format!("{:?}", other),
            })
        }

        // log scopes progressively
        for (scope_idx, scope) in self.table.iter().enumerate() {
            writeln!(f, "scope {}", scope_idx)?;
            // TODO sort scope items before printing, somehow
            for (id, value) in scope {
                // get debug string of this value
                let dbg_ln = match &*value.borrow() {
                    Variable::Owned(node) => format!("{}", prettify(node.to_owned().into())),
                    Variable::Reference(reference) => format!("&{}", prettify(reference.clone())),
                    Variable::Type(instance) => format!("struct {}", prettify(instance.clone())),
                };

                // tab out lines
                let dbg_ln = dbg_ln.replace("\n", "\n\t");

                // print resulting debug line
                writeln!(f, "\t{} = {}", id, dbg_ln)?;
            }
        }
        Ok(())
    }
}
