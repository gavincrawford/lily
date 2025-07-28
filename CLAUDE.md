# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based programming language implementation called "Lily" (`.ly` files). The project consists of two main crates:

- **lylib** - Core language library containing lexer, parser, and interpreter
- **ly** - CLI executable that uses lylib to execute Lily programs

## Development Commands

### Build
```bash
cargo build --verbose          # Build all crates
cargo build -p lylib           # Build library only
cargo build -p ly              # Build CLI only
```

### Testing
```bash
cargo test --verbose           # Run all tests
cargo test -p lylib            # Run library tests only
cargo test [TESTNAME]          # Run specific test containing TESTNAME
```

### Benchmarks
```bash
cargo bench                    # Run all benchmarks (criterion-based)
```

### Running Lily Programs
```bash
cargo run -- <file.ly>                # Run a Lily program
cargo run -- <file.ly> --nostd        # Run without standard library
cargo run -- <file.ly> --debugast     # Debug mode - prints AST during execution
```

## Architecture

### Core Components

1. **Lexer** (`lylib/src/lexer/`) - Tokenizes Lily source code into tokens
2. **Parser** (`lylib/src/parser/`) - Converts tokens into Abstract Syntax Tree (AST)
3. **Interpreter** (`lylib/src/interpreter/`) - Executes the AST

### Key Architectural Details

- **Memory Management**: Uses `SVTable` (Scope-Variable Table) with reference counting (`Rc<RefCell<>>`)
- **Variable System**: Supports scoped variables, modules, and function execution contexts
- **Built-ins**: Standard functions like `print`, `len`, `sort`, `chars` in `interpreter/builtins.rs`
- **Standard Library**: Located in `ly/src/std/` (currently only contains `math.ly`)

### Module Structure

- **lylib/src/interpreter/mod.rs**: Interpreter implementation, executes syntax trees
- **lylib/src/interpreter/mem/**: Memory management subsystem with variable tracking and scope tables
- **lylib/src/interpreter/tests/**: Extensive test suite organized by feature/builtin/implementation categories
- **lylib/src/parser/mod.rs**: Parser implementation, converts tokens into a syntax tree
- **lylib/src/parser/astnode.rs**: AST node variant definitions
- **ly/src/execute.rs**: Main execution logic for the CLI

## Language Features (Lily)

Based on test files, Lily supports:
- Functions with parameters and return values
- Control flow (if/else, loops)
- Variables and scoping
- Structs and constructors
- String operations and indexing
- Mathematical operations
- Import system for modules
- Built-in functions for common operations
- External function support for accessible Rust integration

## Macros System

The project uses an extensive macro system (`lylib/src/macros.rs`) to simplify AST construction and testing:

### Core AST Macros

- **`lit!()`** - Creates literal AST nodes: `lit!(42)`, `lit!(Token::Str("hello"))`
- **`ident!()`** - Creates identifier literals: `ident!("variable_name")`
- **`block!()`** - Creates block AST nodes: `block!(node1, node2, node3)`
- **`node!()`** - Comprehensive AST node creation with multiple patterns:
  - Operations: `node!(op lhs, Token::Add, rhs)`
  - Declarations: `node!(declare x => lit!(42))`
  - Assignments: `node!(assign x => lit!(100))`
  - Functions: `node!(func foo(a, b) => body)`
  - Lists: `node!([lit!(1), lit!(2), lit!(3)])`
  - Indices: `node!(list[0])` or `node!(index target, lit!(0))`

### Testing Macros

- **`parse_eq!()`** - Tests parser output against expected AST
- **`interpret!()`** - Executes `.ly` files and captures output for testing
- **`test!()`** - Comprehensive test macro with two modes:
  - Variable equality: `test!(filename => (var := expected_value))`
  - Output testing: `test!(filename => "expected output")`

### Test Assertion Macros

- **`var_eq_literal!()`** - Compares interpreter variables with literal tokens
- **`var_eq!()`** - Compares interpreter variables with AST nodes

### Built-in Function Macro

- **`exfn!()`** - Defines external/built-in functions in `interpreter/builtins.rs`

### Operator Matching Macro

- **`opmatch!()`** - Pattern matching for binary operations in the interpreter (inline macro in `interpreter/mod.rs`)

## Testing Strategy

Tests are organized into three categories:
- **Feature tests**: Core language functionality
- **Builtin tests**: Standard library functions  
- **Implementation tests**: Complex algorithms (fibonacci, binary search, etc.)

All tests use `.ly` files executed by the interpreter to verify correctness. The macro system enables concise test definitions that automatically handle parsing, execution, and result comparison. All tests are organized by prefix-- for example, testing list indices that dangle should be labeled as `indices_dangling.ly`.

## Code Commenting Conventions

The codebase follows consistent Rust commenting patterns:

### Documentation Comments (`///`)
- **Public functions**: Use `///` for rustdoc documentation with clear purpose descriptions
- **Format**: Start with a verb in present tense (e.g., "Creates", "Returns", "Parses")
- **Examples**: 
  - `/// Creates a new lexer.`
  - `/// Parses all tokens into a program.`
  - `/// Returns the truthiness of this node.`

### Module-level Documentation (`//!`)
- **Purpose**: Describe the module's overall functionality and scope
- **Placement**: At the top of module files after imports
- **Examples**:
  - `//! The lexer breaks down text information into tokens, which can be used to assemble syntax.`
  - `//! A collection of macros that make writing tests easier and slimmer.`

### Inline Comments (`//`)
- **Usage**: Explain complex logic, group related code sections, or clarify non-obvious behavior
- **Style**: 
  - Use lowercase unless required to denote structures or other parts of code that may contain capital letters
  - Use descriptive section headers: `// operators`, `// equalities`, `// keywords and identifiers`
  - Explain the "why" not just the "what": `// if the register contains a keyword, that takes priority`
  - Clarify complex operations: `// convert nodes to variables and make new list`

### Comment Placement Guidelines
- Place comments directly above the code they describe
- Use section comments to group related functionality (operators, data types, etc.)
- Add explanatory comments for non-trivial logic or edge cases
- Document struct fields and enum variants for clarity
