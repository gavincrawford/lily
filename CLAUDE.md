# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based programming language implementation called "Lily" (`.ly` files). See @SYNTAX.md for the language syntax reference. The project consists of two main crates:

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
cargo test --verbose           # Run all tests (verbose)
cargo test -p lylib            # Run library tests only
cargo test [TESTNAME]          # Run specific test containing TESTNAME
```

### Benchmarks
```bash
cargo bench                    # Run all benchmarks (criterion-based)
```

### Running Lily Programs
```bash
cargo run -- <file.ly>                 # Run a Lily program
cargo run -- <file.ly> --no-std        # Run without standard library
cargo run -- <file.ly> --debug-parser  # Debug mode - prints AST during execution
cargo run -- <file.ly> --debug-lexer   # Debug mode - prints tokens during execution
```

## Architecture

### Core Components

1. **Lexer** (`lylib/src/lexer/`) - Tokenizes Lily source code into tokens
2. **Parser** (`lylib/src/parser/`) - Converts tokens into Abstract Syntax Tree (AST)
3. **Interpreter** (`lylib/src/interpreter/`) - Executes the AST

### Key Architectural Details

- **Memory Management**: Uses `SVTable` (Scope-Variable Table) with reference counting (`Rc<RefCell<>>`) and string interning for efficient identifier storage. Each scope frame is a `FlatRcMap<Variable>` — a `Vec`-backed flat map indexed directly by interned identifier (`usize`), avoiding hashing on the hot path. Modules within a scope are still kept in an `FxHashMap`.
- **Copy-on-write SVTable cloning**: `SVTable::clone` is a *shallow* clone that shares variable `Rc`s with the original. This is sound because cloning only happens when instantiating structs from their `template` (constant post-parse), and writes always go through `assign`, which replaces the `Rc` slot rather than mutating through it.
- **String Interning**: Global `StringInterner` deduplicates strings, using `usize` indices for fast lookups
- **Variable System**: Supports scoped variables, modules, and function execution contexts
- **Source position tracking**: The lexer produces `Vec<SpannedToken>` via `Lexer::lex_spanned` — each token carries its source line plus a byte-offset span `[start, end)` into the source buffer. The parser consumes spanned tokens directly and uses `peek_line` to attach line numbers to errors via `anyhow::Context`; byte offsets are stored but not yet surfaced in error messages (planned for the upcoming `LilyError` redesign). The position info is stripped at the AST construction boundary, so runtime values never carry parse-time position data.
- **Structured errors**: `lylib/src/errors.rs` defines `thiserror`-based enums (`MemoryError`, `ExternalFunctionError`, `ParserError`) used internally; these bubble up into `anyhow::Result` at the public API boundary. `ParserError` wraps each parser-statement kind (`Import`, `Declaration`, `Conditional`, `FunctionDecl`, `StructDecl`, `While`, `Return`) around the underlying `anyhow::Error` as a `#[source]`, plus an `Other` catchall with a `From<anyhow::Error>` impl so `?` keeps working inside parser functions.
- **Built-ins**: The `Builtins` struct (`interpreter/builtins.rs`) owns a `Vec<(Symbol, Box<ExFn>)>` of all builtin closures, built once in `Builtins::new()`. `Interpreter::register_builtins` (called from `Interpreter::new`) declares each closure into the base scope as `Variable::Builtin(index)`, where `index` points into that vec. Calling a builtin looks up the closure by `index` at call time rather than storing an `Rc<ExFn>` per variable. External consumers add their own closures via `Interpreter::inject_builtin`, which pushes onto the same vec. Standard functions:
  - `print` - Outputs values to stdout
  - `len` - Returns length of lists or strings
  - `sort` - Sorts lists of numbers or strings (validates type consistency; errors on mixed types)
  - `split` - Splits a string by a `char` or `str` delimiter into a list of strings
  - `chars` - Converts strings to character lists
  - `assert` - Validates conditions, errors if false
  - `sin` / `cos` - Trigonometric sine/cosine of a number (radians)
- **Standard Library**: Located in `ly/src/std/` — `math.ly` (constants `PI`/`E`/`TAU`/`PHI`; functions `max`/`min`/`abs`/`trunc`/`exp`/`acos`/`asin`/`cosh`/`sinh`) and `complex.ly` (`Complex` struct with `add`/`sub`/`mul`/`div`/`as_string`/`mag`). The CLI (`ly/src/execute.rs`) auto-includes both as modules `math` and `complex` on every run unless `--no-std` is passed, so no explicit `import` is needed to use them.
- **Rust edition**: `lylib` is on the 2024 edition. Notable deps: `anyhow`, `thiserror`, `derivative` (used to derive `PartialEq`/`Debug` on `ASTNode` while `#[derivative(PartialEq = "ignore")]`-ing fields like struct templates and module paths), and `rustc-hash` (`FxHashMap`).

### Module Structure

- **lylib/src/interpreter/mod.rs**: Interpreter implementation, executes syntax trees
- **lylib/src/interpreter/builtins.rs**: Built-in function definitions (print, len, sort, split, chars, assert)
- **lylib/src/interpreter/execute_function.rs**: Function execution logic
- **lylib/src/interpreter/node_to_id.rs**: Converts AST nodes to identifiers
- **lylib/src/interpreter/resolve_refs.rs**: Resolves references in lists (indices and nested lists)
- **lylib/src/interpreter/mem/**: Memory management subsystem with variable tracking and scope tables
- **lylib/src/interpreter/mem/svtable.rs**: `SVTable` (scoped variable table) — vec of `FlatRcMap<Variable>` scope frames + module map; implements the `MemoryInterface` trait
- **lylib/src/interpreter/mem/flatrcmap.rs**: `FlatRcMap<T>` — generic `Vec`-backed map indexed by interned `usize` IDs, used for scope frames; supports `shallow_clone` for COW
- **lylib/src/interpreter/mem/variable.rs**: `Variable` enum (`Owned`/`Function`/`Builtin`/`Type`); `Builtin(index)` references a closure by index into `Interpreter`'s `Builtins` vec rather than holding an `Rc<ExFn>` directly
- **lylib/src/interpreter/mem/drop.rs**: Custom `Drop` glue for memory cleanup
- **lylib/src/interpreter/id/**: Identifier class declaration and associated functions
- **lylib/src/interpreter/tests/**: Extensive test suite organized by feature/builtin/implementation categories
- **lylib/src/errors.rs**: Shared `thiserror`-derived error enums (`MemoryError`, `ExternalFunctionError`, `ParserError`)
- **lylib/src/interner.rs**: String interning system for memory optimization
- **lylib/src/lexer/mod.rs**: Lexer implementation, converts a buffer into tokens; exposes `lex` (raw tokens) and `lex_spanned` (tokens with line + byte-offset span)
- **lylib/src/lexer/token/mod.rs**: `Token` enum and `SpannedToken` (token + line + byte-offset span `[start, end)`); `Token::at(line, start, end)` produces a `SpannedToken`
- **lylib/src/parser/mod.rs**: Parser implementation, consumes `Vec<SpannedToken>` into a syntax tree (uses `VecDeque` internally; `peek_line` attaches line context to errors). Postfix `++`/`--` are desugared at parse time into `ASTNode::Assign { target, value: Op(target, +/-, 1) }`, so they work on any valid assignment target (simple identifiers, index expressions, deref chains). Compound assignment `+=`/`-=` desugars the same way via `parse_compound_assign` (`ASTNode::Assign { target, value: Op(target, +/-, rhs) }`), with `rhs` parsed as a full expression.
- **lylib/src/parser/astnode.rs**: AST node variant definitions (includes `Break`, `UnaryOp`, `Identifier`, etc.). `Identifier(ID)` is distinct from `Literal(Token)`: the parser and interpreter treat identifiers and literals as separate node kinds.
- **lylib/src/execute.rs**: `LyConfig` factory — configures and runs the interpreter (debug toggles, includes/imports)
- **ly/src/main.rs**: CLI entry point
- **ly/src/execute.rs**: CLI-side execution glue
- **ly/src/std/**: Bundled standard library `.ly` files

## Macros System

The project uses an extensive macro system (`lylib/src/macros.rs`) to simplify AST construction and testing:

### Core AST Macros

- **`intern!()`** - Converts string to interned identifier: `intern!("variable_name")`
- **`resolve!()`** - Resolves interned identifier back to string: `resolve!(id)`
- **`lit!()`** - Creates literal AST nodes: `lit!(42)`, `lit!(Token::Str("hello"))`
- **`ident!()`** - Creates identifier nodes (`ASTNode::Identifier`): `ident!("variable_name")`
- **`block!()`** - Creates block AST nodes: `block!(node1, node2, node3)`
- **`node!()`** - Comprehensive AST node creation with multiple patterns:
  - Operations: `node!(op lhs, Token::Add, rhs)`
  - Unary operations: `node!(unary Token::Sub, ident!("x"))` (for `-x`/`!x`; postfix `++`/`--` and compound `+=`/`-=` desugar to `Assign` at parse time)
  - Declarations: `node!(declare x => lit!(42))`
  - Assignments: `node!(assign x => lit!(100))`
  - Functions: `node!(func foo(a, b) => body)`
  - Function calls: `node!(foo(arg1, arg2))` or `node!(a.b.c(arg))`
  - Conditionals: `node!(if cond => if_body; else => else_body;)`
  - Loops: `node!(loop cond => body;)` and `node!(break)`
  - Returns: `node!(return value)`
  - Modules: `node!(mod name => body)`
  - Structures: `node!(struct Name => body)`
  - Lists: `node!([lit!(1), lit!(2), lit!(3)])`
  - Indices: `node!(list[0])`, `node!(list[expr])`, or `node!(index target, lit!(0))`
  - Derefs: `node!(a.b.c)` or `node!(deref parent, child)`

### Testing Macros

- **`parse_test!()`** (`lylib/src/parser/tests/mod.rs`) - Generates a full parser `#[test]` function comparing parser output against an expected AST, with three modes:
  - Expect-AST: `parse_test!(name => code; block1, block2, ...)`
  - Expect-panic: `parse_test!(name => code; panic)`
  - Modified parser path (for import-relative tests): `parse_test!(name (path) => code; block1, ...)`
- **`test!()`** (`lylib/src/interpreter/tests/mod.rs`) - Comprehensive interpreter test macro that reads and executes a `.ly` file named after the test, with three modes:
  - Variable equality: `test!(filename => (var := expected_value))`
  - Output testing: `test!(filename => "expected output")`
  - Panic/error testing: `test!(filename => panic)` - expects the test to fail

### Built-in Function Helper

- **`unpack!()`** - Local macro in `interpreter/builtins.rs` that destructures a builtin's `&Vec<Rc<ASTNode>>` argument slice into named bindings (e.g. `unpack!(args => a, b)`), returning `ExternalFunctionError::InvalidArguments` on arity mismatch. Builtin closures are added directly as `(Symbol, Box<ExFn>)` entries in `Builtins::new()`'s vec, not via a dedicated declaration macro.

### Operator Matching Macro

- **`opmatch!()`** - Pattern matching for binary operations in the interpreter (inline macro in `interpreter/mod.rs`)

## Testing Strategy

Tests are organized into three categories:
- **Feature tests**: Core language functionality
- **Builtin tests**: Standard library functions  
- **Implementation tests**: Complex algorithms (fibonacci, binary search, etc.)

All tests use `.ly` files executed by the interpreter to verify correctness. The macro system enables concise test definitions that automatically handle parsing, execution, and result comparison. All tests are organized by prefix-- for example, testing list indices that dangle should be labeled as `indices_dangling.ly`.

A dedicated test (`interpreter::tests::syntax` in `lylib/src/interpreter/tests/mod.rs`) runs every ` ```lily ` code block in `SYNTAX.md` through a bare `Interpreter` (no bundled stdlib included), failing the build if any block fails to lex, parse, or execute. Mark a block ` ```lily !skip ` to exclude it from this check — use this for illustrative snippets that reference files that don't exist or that rely on the CLI's auto-included `math`/`complex` modules, which the bare interpreter used by this test doesn't have.

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
