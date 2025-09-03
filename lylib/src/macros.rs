//! A collection of macros for simplifying the process of making `ASTNode`s.
//! The macros here are primarily used for testing, where we compare against large hand-made
//! `ASTNode` trees, or anywhere we need to instantiate nodes easily.

/// Converts a string to an interned identifier.
macro_rules! intern {
    ($id:expr) => {{
        let mut i = crate::get_global_interner().unwrap();
        let id = i.intern($id);
        drop(i);
        id
    }};
}

/// Resolves an interned identifier backwards to the original string.
macro_rules! resolve {
    ($id:expr) => {{
        let i = crate::get_global_interner().unwrap();
        let id = i.resolve($id).to_owned();
        drop(i);
        id
    }};
}

/// Shorthand for creating a literal.
macro_rules! lit {
    // numbers
    ($literal:literal) => {
        lit!($literal.into())
    };

    // literals
    ($token:expr) => {
        Rc::new(ASTNode::Literal($token))
    };
}

/// Shorthand for creating a literal identifier. Used for testing.
#[cfg(test)]
macro_rules! ident {
    ($id:expr) => {
        lit!(Token::Identifier(intern!($id)))
    };
}

/// Shorthand for creating node blocks. Used for testing.
#[cfg(test)]
macro_rules! block {
    ($($node:expr),*) => {{
        let block = vec![$($node),*];
        ASTNode::Block(block).into()
    }};
}

/// Shorthand for all AST nodes. Used for testing.
/// # Usage
/// ```
/// // declare/assign variables
/// node!(declare var => lit!(1));
/// node!(assign var => lit!(2));
///
/// // operators
/// node!(op lit!(1), Token::Add, lit!(2));
///
/// // conditionals
/// node!(if lit!(true) => block!(lit!(1)); else => block!(lit!(2)););
///
/// // function calls
/// node!(print(lit!(42)));
/// node!(math.pow(lit!(2), lit!(3))); // module function call
///
/// // function definitions
/// node!(func add(a, b) => block!(node!(return node!(op ident!("a"), Token::Add, ident!("b")))));
///
/// // modules
/// node!(mod math => block!(node!(func pow(base, exp) => block!())));
///
/// // structures
/// node!(struct Point => block!(node!(declare x => lit!(0)), node!(declare y => lit!(0))));
///
/// // lists
/// node!([lit!(1), lit!(2), lit!(3)]);
///
/// // indexing
/// node!(list[0]); // literal index
/// node!(list[ident!("i")]); // expression index
/// node!(index ident!("list"), 0); // explicit index syntax
///
/// // dereferencing (accessing properties)
/// node!(obj.field);
/// node!(obj.nested.field);
///
/// // return statements
/// node!(return lit!(42));
/// ```
#[cfg(test)]
macro_rules! node {
    // operators (`op lit!(1), Add, lit!(1)`)
    (op $lhs:expr, $op:expr, $rhs:expr) => {
        Rc::new(ASTNode::Op {
            lhs: $lhs,
            op: $op,
            rhs: $rhs,
        })
    };

    // unary operators (`unary Decrement, ident!(..)`)
    (unary $op:expr, $target:expr) => {
        Rc::new(ASTNode::UnaryOp {
            target: $target,
            op: $op,
        })
    };

    // declarations & assignments
    (declare $id:tt => $val:expr) => { // implied (`declare x => lit!(..)`)
        node!(declare ident!(stringify!($id)) => $val)
    };
    (declare $id:expr => $val:expr) => { // literal (`declare ident!("x") => lit!(..)`)
        ASTNode::Declare {
            target: $id,
            value: $val,
        }
        .into()
    };
    (assign $id:tt => $val:expr) => { // implied (`assign x => lit!(..)`)
        node!(assign ident!(stringify!($id)) => $val)
    };
    (assign $id:expr => $val:expr) => { // literal (`assign ident!("x") => lit!(..)`)
        ASTNode::Assign {
            target: $id,
            value: $val,
        }
        .into()
    };

    // conditionals (`if node!(..) => block!(..); else => block!(..);`)
    (if $cond:expr => $ifbody:expr; else => $elsebody:expr;) => {
        ASTNode::Conditional {
            condition: $cond,
            if_body: $ifbody,
            else_body: $elsebody,
        }.into()
    };

    // function calls
    (call $fn:expr) => { // literal calls (`call node!(..)`) *no args*
        ASTNode::FunctionCall {
            target: $fn,
            arguments: vec![],
        }.into()
    };
    ($fn:tt($($arg:expr),*)) => { // implied calls (`a()`)
        ASTNode::FunctionCall {
            target: ident!(stringify!($fn)),
            arguments: vec![$($arg),*],
        }.into()
    };
    ($first:tt $(. $rest:tt)+ ($($arg:expr),*)) => { // deref-fn calls (`a.b.c()`)
        ASTNode::FunctionCall {
            target: node!($first $(. $rest)+),
            arguments: vec![$($arg),*],
        }.into()
    };

    // function declarations
    (func $fn:tt($($arg:tt),*) => $body:expr) => {
        ASTNode::Function {
            id: intern!(stringify!($fn)).as_id(),
            arguments: vec![$(intern!(stringify!($arg))),*],
            body: $body,
        }.into()
    };

    // return statements (`return node!(..)`)
    (return $value:expr) => {
        ASTNode::Return($value).into()
    };

    // modules (`mod xyz => block!(..)`)
    (mod $id:tt => $body:expr) => {
        ASTNode::Module {
            alias: Some(intern!(stringify!($id)).into()),
            body: $body,
        }.into()
    };

    // structures (`struct XYZ => block!(..)`)
    (struct $id:tt => $body:expr) => {
        ASTNode::Struct {
            id: intern!(stringify!($id)).as_id(),
            body: $body,
        }.into()
    };

    // lists (`[1, 2, 3]`)
    ([$($item:expr),*]) => {
        {
            // convert nodes to variables and make new list
            let values: Vec<Rc<ASTNode>> = vec![$($item),*];
            let mut variables = vec![];
            for value in values {
                let variable: Variable = value.into();
                variables.push(variable.into());
            }
            ASTNode::List(variables).into()
        }
    };

    // indices (`list[0]`)
    (index $list:expr, $idx:expr) => {
        // explicit index
        ASTNode::Index {
            target: $list,
            index: lit!($idx),
        }
        .into()
    };
    ($list:ident[$idx:literal]) => {
        // numerical index
        ASTNode::Index {
            target: ident!(stringify!($list)),
            index: lit!($idx),
        }
        .into()
    };
    ($list:ident[$idx:expr]) => {
        // expression index
        ASTNode::Index {
            target: ident!(stringify!($list)),
            index: $idx,
        }
        .into()
    };

    // derefs
    (deref $parent:expr, $child:expr) => {{ // literal derefs (`deref node!(..), ident!(..)`)
        ASTNode::Deref {
            parent: $parent,
            child: $child,
        }.into()
    }};
    ($first:tt $(. $rest:tt)+) => {{ // implied derefs (`a.b.c`)
        let mut current = ident!(stringify!($first));
        $(
            current = ASTNode::Deref {
                parent: current,
                child: ident!(stringify!($rest)),
            }.into();
        )+
        current
    }};
}
