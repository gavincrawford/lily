//! A collection of macros that make writing tests easier and slimmer.

/// Shorthand for creating and executing the parser, and comparing its output to an expression.
#[macro_export]
macro_rules! parse_eq {
    ($code:expr; $($block:expr),*) => {
        (|| {
            let result = Parser::new(Lexer::new().lex($code.into()).unwrap()).parse();
            assert!(result.is_ok(), "Parser failed: {:?}", result);
            assert_eq!(result.unwrap(), block!($($block),*));
        })()
    };
    ($code:expr, $path:expr; $($block:expr),*) => {
        (|| {
            let mut parser = Parser::new(Lexer::new().lex($code.into()).unwrap());
            parser.set_pwd($path.into());
            let result = parser.parse();
            assert!(result.is_ok(), "Parser failed: {:?}", result);
            assert_eq!(result.unwrap(), block!($($block),*));
        })()
    };
}

/// Shorthand for executing test code located at the provided path.
#[macro_export]
macro_rules! interpret {
    ($path:expr) => {{
        // interpret file
        use std::io::Cursor;
        let mut i = Interpreter::new(Cursor::new(vec![]), Cursor::new(vec![]));
        let mut p = Parser::new(Lexer::new().lex(include_str!($path).to_string()).unwrap());
        p.set_pwd(std::path::PathBuf::from("src/interpreter/tests/feature/"));
        let ast = p.parse().unwrap();
        i.execute(ast).unwrap();

        // read output
        let mut buf = String::new();
        let mut output = i.output.borrow_mut();
        output.set_position(0);
        output.read_to_string(&mut buf).unwrap();
        drop(output);

        (i, buf)
    }};
}

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
            let values = vec![$($item),*];
            ASTNode::List(SVTable::new_with(values)).into()
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
