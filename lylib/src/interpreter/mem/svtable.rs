//! Implements the SVTable, or the scoped-variable table.

use super::*;
use crate::get_global_interner;
use anyhow::{bail, Result};
use rustc_hash::FxHashMap;
use std::{cell::RefCell, fmt::Display, rc::Rc, slice::Iter};

/// Scoped-variable table. Holds values with respect to their variable names.
/// Internally, uses a `FxHashMap` providing fast but less secure access.
#[derive(Debug, PartialEq)]
pub struct SVTable {
    /// Holds all the scope frames, each of which hold their respective variables.
    table: Vec<FxHashMap<usize, Rc<RefCell<Variable>>>>,
    /// Holds all the modules defined at this SVTable's scope.
    modules: FxHashMap<usize, Rc<RefCell<SVTable>>>,
}

impl SVTable {
    /// Creates a new scoped-variable table with a default scope.
    pub fn new() -> Self {
        let mut svt = Self {
            table: vec![],
            modules: FxHashMap::default(),
        };
        svt.add_scope();
        svt
    }

    /// Returns the iterator to the internal list of frames.
    pub fn iter(&self) -> Iter<'_, FxHashMap<usize, Rc<RefCell<Variable>>>> {
        self.table.iter()
    }

    /// Returns the inner list of frames.
    pub fn inner(&self) -> &Vec<FxHashMap<usize, Rc<RefCell<Variable>>>> {
        &self.table
    }

    /// Returns the inner list of frames, mutable.
    pub fn inner_mut(&mut self) -> &mut Vec<FxHashMap<usize, Rc<RefCell<Variable>>>> {
        &mut self.table
    }

    /// Adds a new module. Returns a reference to the newly created module.
    pub fn add_module(&mut self, name: usize) -> Rc<RefCell<SVTable>> {
        self.modules
            .insert(name, RefCell::new(SVTable::new()).into());
        self.modules.get(&name).unwrap().to_owned()
    }

    /// Gets a module by name. Returns an immutable reference to the module if found.
    pub fn get_module(&self, name: usize) -> Result<Rc<RefCell<SVTable>>> {
        if let Some(module) = self.modules.get(&name) {
            Ok(module.clone())
        } else {
            let interner = get_global_interner().lock().unwrap();
            let name_str = interner.resolve(name);
            bail!("failed to find module '{}'", name_str);
        }
    }

    /// Adds a new scope.
    pub fn add_scope(&mut self) {
        self.table.push(FxHashMap::default());
    }

    /// Gets a scope map. Mutable by default.
    pub fn get_scope(
        &mut self,
        index: usize,
    ) -> Option<&mut FxHashMap<usize, Rc<RefCell<Variable>>>> {
        self.table.get_mut(index)
    }

    /// Returns the number of scopes in this table.
    pub fn scopes(&self) -> usize {
        self.table.len()
    }
}

impl MemoryInterface for SVTable {
    fn get_owned(&self, id: usize) -> Result<Variable> {
        // find id in any scope and return owned
        for scope in self.iter().rev() {
            if let Some(variable) = scope.get(&id) {
                return Ok((**variable).borrow().clone());
            }
        }

        // if no value is found, bail
        let interner = get_global_interner().lock().unwrap();
        let id_str = interner.resolve(id);
        bail!("failed to get owned value {:?}", id_str)
    }

    fn get_ref(&self, id: usize) -> Result<Rc<RefCell<Variable>>> {
        // find id in any scope and return reference
        for scope in self.iter().rev() {
            if let Some(variable) = scope.get(&id) {
                return Ok((*variable).clone());
            }
        }

        // if no value is found, bail
        let interner = get_global_interner().lock().unwrap();
        let id_str = interner.resolve(id);
        bail!("failed to get owned value {:?}", id_str)
    }

    fn get_module(&self, id: usize) -> Result<Rc<RefCell<SVTable>>> {
        match self.modules.get(&id) {
            Some(module) => Ok(module.clone()),
            _ => {
                let interner = get_global_interner().lock().unwrap();
                let id_str = interner.resolve(id);
                bail!("could not find module '{:?}'", id_str)
            }
        }
    }

    fn declare(&mut self, id: usize, value: Variable, scope: usize) -> Result<()> {
        // add scopes if necessary
        while self.scopes() <= scope {
            self.add_scope();
        }

        // get variable map and insert new value. if the value already exists, bail
        let var_map = self
            .get_scope(scope)
            .context(format!("cannot delcare at scope {scope}",))?;
        if let Some(_) = var_map.insert(id, Rc::new(RefCell::new(value))) {
            let interner = get_global_interner().lock().unwrap();
            let id_str = interner.resolve(id);
            bail!("variable '{}' already exists", id_str);
        }
        Ok(())
    }

    fn assign(&mut self, id: usize, value: Variable, scope: usize) -> Result<()> {
        // replace the value of the top-most variable if possible
        for scope in self.iter().rev() {
            if let Some(variable) = scope.get(&id) {
                *variable.borrow_mut() = value;
                return Ok(());
            }
        }

        // otherwise, manual insert. this is used for structures & modules
        let var_map = self
            .get_scope(scope)
            .context(format!("cannot assign at scope {scope}",))?;
        var_map.insert(id, Rc::new(RefCell::new(value)));
        Ok(())
    }
}

impl Display for SVTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn prettify(node: Rc<ASTNode>) -> String {
            match &*node {
                ASTNode::Literal(Token::Identifier(id)) => id.to_string(),
                ASTNode::Literal(token) => format!("{token:?}"),
                ASTNode::Op { lhs, op, rhs } => format!(
                    "{} {:?} {}",
                    prettify(lhs.clone()),
                    op,
                    prettify(rhs.clone())
                ),
                ASTNode::Block(lines) => lines
                    .iter()
                    .map(|ln| prettify(ln.clone()))
                    .collect::<Vec<String>>()
                    .join(", ")
                    .to_string(),
                ASTNode::Return(value) => prettify(value.clone()),
                ASTNode::Function {
                    id,
                    arguments,
                    body,
                } => format!(
                    "{}({}) => {}",
                    id.to_path_compat().join("."),
                    arguments
                        .iter()
                        .map(|&arg_id| {
                            get_global_interner()
                                .lock()
                                .unwrap()
                                .resolve(arg_id)
                                .to_string()
                        })
                        .collect::<Vec<String>>()
                        .join(", "),
                    prettify(body.clone())
                ),
                other => format!("{other:?}"),
            }
        }

        // log scopes progressively
        for (scope_idx, scope) in self.table.iter().enumerate() {
            // log scope level
            writeln!(f, "scope {scope_idx}")?;

            // iterate through scope values, sorted by key name
            let mut keys = scope.keys().collect::<Vec<&usize>>();
            keys.sort();
            for &key in keys {
                // resolve key to string for display
                let interner = get_global_interner().lock().unwrap();
                let key_str = interner.resolve(key);

                // obtain debug string respective to variable value
                let value = scope.get(&key).unwrap();
                let dbg_ln = match &*value.borrow() {
                    Variable::Owned(node) => prettify(node.to_owned().into()).to_string(),
                    Variable::Function(reference) => format!("&{}", prettify(reference.clone())),
                    Variable::Extern(_) => "EXTERN".to_string(),
                    Variable::Type(instance) => format!("struct {}", prettify(instance.clone())),
                };

                // tab out endlines to keep indents, and print it
                let dbg_ln = dbg_ln.replace("\n", "\n\t");
                writeln!(f, "\t{key_str} = {dbg_ln}")?;
            }
        }
        Ok(())
    }
}
