//! A collection of macros that make writing tests easier and slimmer.

/// Shorthand for creating and executing the parser.
macro_rules! parse {
    ($code:expr) => {
        (|| {
            let result = Parser::new(Lexer::new().lex($code.into()).unwrap()).parse();
            assert!(result.is_ok(), "Parser failed: {:?}", result);
            result
        })()
    };
    ($code:expr, $path:expr) => {
        (|| {
            let mut parser = Parser::new(Lexer::new().lex($code.into()).unwrap());
            parser.set_pwd($path.into());
            let result = parser.parse();
            assert!(result.is_ok(), "Parser failed: {:?}", result);
            result
        })()
    };
}

/// Shorthand for creating a literal.
macro_rules! lit {
    // numbers
    ($literal:literal) => {
        lit!(Token::Number($literal as f32))
    };

    // literals
    ($token:expr) => {
        Rc::new(ASTNode::Literal($token))
    };
}

/// Shorthand for creating a literal identifier.
macro_rules! ident {
    ($id:expr) => {
        lit!(Token::Identifier($id.into()))
    };
}

/// Shorthand for all AST nodes.
macro_rules! node {
    // blocks
    (block $block:expr) => {
        ASTNode::Block($block).into()
    };

    // ops, in two formats
    (op $lhs:expr, $op:expr, $rhs:expr) => {
        ASTNode::Op {
            lhs: $lhs,
            op: $op,
            rhs: $rhs,
        }
        .into()
    };

    // declarations & assignments
    (declare $id:expr => $val:expr) => {
        ASTNode::Declare {
            target: $id,
            value: $val,
        }
        .into()
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
    ($list:ident[$idx:expr]) => {
        ASTNode::Index {
            target: ident!(stringify!($list)),
            index: lit!(Token::Number($idx as f32)),
        }
        .into()
    };
}
