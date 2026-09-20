Lily is an interpreted language that has been a personal learning project of mine for the last two years. To see an overview of how Lily executes files, see [the Git diagram here](https://gitdiagram.com/gavincrawford/lily).

## Building

Lily is written in Rust, so you'll need a [Rust toolchain](https://rustup.rs/). (the `lylib` crate uses the 2024 edition)
To compile Lily, run the following command inside of the cloned repository:

```bash
cargo build --release
```

This produces the `ly` executable at `target/release/ly`.

## Running

To execute a program, pass a Lily source file (`.ly`) to the CLI:

```bash
cargo run --release -- program.ly

# or, using the built binary directly
./target/release/ly program.ly
```

For a tour of the language syntax, see [SYNTAX.md](SYNTAX.md).

### Flags

| Flag | Description |
|------|-------------|
| `--no-std` | Run without the bundled standard library modules (`math` and `complex`), which are otherwise included on every run |
| `--debug-lexer` | Print the lexer's tokens during execution |
| `--debug-parser` | Print the parser's AST during execution |
| `--debug-memory` | Print the memory map after execution |
| `-h`, `--help` | Print usage information |
| `-V`, `--version` | Print the CLI version |

When running through Cargo, flags go after the `--` separator:

```bash
cargo run -- program.ly --no-std
```
