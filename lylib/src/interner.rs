//! String interning system for efficient variable name storage.
//!
//! The interner stores each unique string once and maps it to a usize identifier.
//! This reduces memory usage and makes string comparisons faster (comparing usize vs String).

use rustc_hash::FxHashMap;

/// String interner that maps strings to usize identifiers.
///
/// Uses a Vec for storage indexed by usize, and a HashMap for reverse lookup.
#[derive(Debug, Clone)]
pub struct StringInterner {
    /// Storage for interned strings, indexed by their interned ID
    strings: Vec<String>,
    /// Map from string to interned ID for fast lookup during interning
    indices: FxHashMap<String, usize>,
}

impl StringInterner {
    /// Creates a new empty string interner.
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            indices: FxHashMap::default(),
        }
    }

    /// Interns a string and returns its unique identifier.
    ///
    /// If the string is already interned, returns the existing identifier.
    /// Otherwise, allocates a new identifier and stores the string.
    pub fn intern(&mut self, string: impl Into<String>) -> usize {
        let string = string.into();
        if let Some(&id) = self.indices.get(&string) {
            // string already interned, return existing ID
            id
        } else {
            // new string, allocate new ID
            let id = self.strings.len();
            self.indices.insert(string.clone(), id);
            self.strings.push(string);
            id
        }
    }

    /// Resolves an interned identifier back to its string.
    ///
    /// Returns the string associated with the given identifier.
    /// Panics if the identifier is invalid.
    pub fn resolve(&self, id: usize) -> &str {
        self.strings
            .get(id)
            .map(|s| s.as_str())
            .unwrap_or_else(|| panic!("Invalid interned string ID: {}", id))
    }

    /// Returns the number of interned strings.
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    /// Returns true if the interner is empty.
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    /// Returns an iterator over all interned strings with their IDs.
    pub fn iter(&self) -> impl Iterator<Item = (usize, &str)> {
        self.strings
            .iter()
            .enumerate()
            .map(|(id, string)| (id, string.as_str()))
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_interning() {
        let mut interner = StringInterner::new();

        let id1 = interner.intern("hello".to_string());
        let id2 = interner.intern("world".to_string());
        let id3 = interner.intern("hello".to_string()); // duplicate

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 0); // same as id1

        assert_eq!(interner.resolve(id1), "hello");
        assert_eq!(interner.resolve(id2), "world");
        assert_eq!(interner.len(), 2);
    }

    #[test]
    fn test_empty_interner() {
        let interner = StringInterner::new();
        assert_eq!(interner.len(), 0);
        assert!(interner.is_empty());
    }

    #[test]
    fn test_iterator() {
        let mut interner = StringInterner::new();
        interner.intern("foo".to_string());
        interner.intern("bar".to_string());
        interner.intern("baz".to_string());

        let collected: Vec<(usize, &str)> = interner.iter().collect();
        assert_eq!(collected, vec![(0, "foo"), (1, "bar"), (2, "baz")]);
    }

    #[test]
    #[should_panic(expected = "Invalid interned string ID: 999")]
    fn test_invalid_resolve() {
        let interner = StringInterner::new();
        interner.resolve(999);
    }

    #[test]
    fn test_intern_empty_string() {
        let mut interner = StringInterner::new();
        let id = interner.intern("".to_string());
        assert_eq!(interner.resolve(id), "");
    }

    #[test]
    fn test_intern_complex_strings() {
        let mut interner = StringInterner::new();

        let variables = vec![
            "variable_name".to_string(),
            "some.member.access".to_string(),
            "function_123".to_string(),
            "_private_var".to_string(),
        ];

        let mut ids = Vec::new();
        for var in &variables {
            ids.push(interner.intern(var.clone()));
        }

        // verify all strings can be resolved correctly
        for (i, var) in variables.iter().enumerate() {
            assert_eq!(interner.resolve(ids[i]), var);
        }

        // verify re-interning returns same IDs
        for (i, var) in variables.iter().enumerate() {
            assert_eq!(interner.intern(var.clone()), ids[i]);
        }
    }
}
