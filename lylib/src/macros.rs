//! A collection of macros for simplifying the process of making `ASTNode`s.
//! The macros here are primarily used for testing, where we compare against large hand-made
//! `ASTNode` trees, or anywhere we need to instantiate nodes easily.

/// Shorthand for creating a literal.
#[macro_export]
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

/// Shorthand for creating a literal identifier.
#[macro_export]
macro_rules! ident {
    ($id:expr) => {
        lit!(Token::Identifier($id.into()))
    };
}

/// Shorthand for creating node blocks.
#[macro_export]
macro_rules! block {
    ($($node:expr),*) => {{
        let block = vec![$($node),*];
        ASTNode::Block(block).into()
    }};
}

/// Shorthand for all AST nodes.
#[macro_export]
macro_rules! node {
    // ops, in two formats
    (op $lhs:expr, $op:expr, $rhs:expr) => {
        Rc::new(ASTNode::Op {
            lhs: $lhs,
            op: $op,
            rhs: $rhs,
        })
    };

    // declarations & assignments
    (declare $id:tt => $val:expr) => {
        node!(declare ident!(stringify!($id)) => $val)
    };
    (declare $id:expr => $val:expr) => {
        ASTNode::Declare {
            target: $id,
            value: $val,
        }
        .into()
    };
    (assign $id:tt => $val:expr) => {
        node!(assign ident!(stringify!($id)) => $val)
    };
    (assign $id:expr => $val:expr) => {
        ASTNode::Assign {
            target: $id,
            value: $val,
        }
        .into()
    };

    // conditionals
    (if $cond:expr => $ifbody:expr; else => $elsebody:expr;) => {
        ASTNode::Conditional {
            condition: $cond,
            if_body: $ifbody,
            else_body: $elsebody,
        }.into()
    };

    // functions
    ($($fn:tt).+($($arg:expr),*)) => {
        ASTNode::FunctionCall {
            target: ident!(stringify!($($fn).+)),
            arguments: vec![$($arg),*],
        }.into()
    };
    (func $fn:tt($($arg:tt),*) => $body:expr) => {
        ASTNode::Function {
            id: ID::new(stringify!($fn)),
            arguments: vec![$(String::from(stringify!($arg))),*],
            body: $body,
        }.into()
    };
    (return $value:expr) => {
        ASTNode::Return($value).into()
    };

    // modules
    (mod $id:tt => $body:expr) => {
        ASTNode::Module {
            alias: Some(stringify!($id).into()),
            body: $body,
        }.into()
    };

    // structures
    (struct $id:expr => $body:expr) => {
        ASTNode::Struct {
            id: ID::new(stringify!($id)),
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
}
