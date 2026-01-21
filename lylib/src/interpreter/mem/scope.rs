use super::{ASTNode, Rc, RefCell, Token, Variable};

#[derive(Default, Debug, PartialEq)]
pub struct Scope {
    map: Vec<Rc<RefCell<Variable>>>,
}

impl Scope {
    fn new(map: Vec<Rc<RefCell<Variable>>>) -> Self {
        Self { map }
    }

    pub(crate) fn deep_clone(&self) -> Scope {
        // deep clone the values within this scope
        let deep_map = self
            .map
            .iter()
            .map(|var| Rc::new(RefCell::new(var.borrow().clone())))
            .collect();

        Scope::new(deep_map)
    }

    /// Gets a reference to a variable by its identifier.
    pub(crate) fn get(&self, id: usize) -> Option<&Rc<RefCell<Variable>>> {
        self.map.get(id).and_then(|var| {
            // treat Undefined as non-existent variable
            if *var.borrow() == Variable::Owned(ASTNode::Literal(Token::Undefined)) {
                None
            } else {
                Some(var)
            }
        })
    }

    /// Inserts a variable into the scope, returning the previous value if it existed.
    pub(crate) fn insert(
        &mut self,
        id: usize,
        var: Rc<RefCell<Variable>>,
    ) -> Option<Rc<RefCell<Variable>>> {
        if id < self.map.len() {
            // replace existing value and return the old one
            Some(std::mem::replace(&mut self.map[id], var))
        } else {
            // fill gaps with Undefined
            while id > self.map.len() {
                self.map
                    .push(Rc::new(RefCell::new(Variable::Owned(ASTNode::Literal(
                        Token::Undefined,
                    )))));
            }
            // push new value at the end
            self.map.push(var);
            None
        }
    }

    /// Removes all variables from this scope.
    pub(crate) fn clear(&mut self) {
        self.map.clear();
    }

    /// Returns a list of the variables within this scope, by ID.
    pub(crate) fn keys(&self) -> Vec<usize> {
        let mut keys = vec![];
        for (idx, var) in self.map.iter().enumerate() {
            if *var.borrow() != Variable::Owned(ASTNode::Literal(Token::Undefined)) {
                keys.push(idx);
            }
        }
        keys
    }
}
