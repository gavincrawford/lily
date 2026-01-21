use super::{Rc, RefCell, Variable};
use rustc_hash::FxHashMap;

#[derive(Default, Debug, PartialEq)]
pub struct Scope {
    map: FxHashMap<usize, Rc<RefCell<Variable>>>,
}

impl Scope {
    fn new(map: FxHashMap<usize, Rc<RefCell<Variable>>>) -> Self {
        Self { map }
    }

    pub(crate) fn deep_clone(&self) -> Scope {
        // deep clone the values within this scope
        let deep_map = self
            .map
            .iter()
            .map(|(&id, var)| (id, Rc::new(RefCell::new(var.borrow().clone()))))
            .collect();

        Scope::new(deep_map)
    }

    /// Gets a reference to a variable by its identifier.
    pub(crate) fn get(&self, id: &usize) -> Option<&Rc<RefCell<Variable>>> {
        self.map.get(id)
    }

    /// Inserts a variable into the scope, returning the previous value if it existed.
    pub(crate) fn insert(
        &mut self,
        id: usize,
        var: Rc<RefCell<Variable>>,
    ) -> Option<Rc<RefCell<Variable>>> {
        self.map.insert(id, var)
    }

    /// Removes all variables from this scope.
    pub(crate) fn clear(&mut self) {
        self.map.clear();
    }

    /// Returns a list of the variables within this scope, by ID.
    pub(crate) fn keys(&self) -> Vec<&usize> {
        self.map.keys().collect::<Vec<&usize>>()
    }
}
