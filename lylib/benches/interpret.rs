use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use lylib::{
    interpreter::Interpreter,
    lexer::Lexer,
    parser::{ASTNode, Parser},
};
use std::{
    hint::black_box,
    io::{self, Cursor},
    rc::Rc,
};

macro_rules! ast {
    ($src:expr) => {
        black_box(
            Parser::new(Lexer::new().lex(include_str!($src).to_string()).unwrap())
                .parse()
                .unwrap(),
        )
    };
}

fn criterion_benchmark(c: &mut Criterion) {
    fn interpret(ast: Rc<ASTNode>) {
        // Direct in/out to in-memory locations to avoid introducing system instability
        let mut i = Interpreter::new(Cursor::new(""), io::sink());
        i.execute(ast).expect("failed to execute benchmark AST.");
    }

    c.bench_function("fibonacci", |b| {
        b.iter_batched(
            || ast!("../src/interpreter/tests/implementation/fibonacci.ly"),
            |ast| interpret(ast),
            BatchSize::LargeInput,
        )
    });
    c.bench_function("matrix rotation", |b| {
        b.iter_batched(
            || ast!("../src/interpreter/tests/implementation/matrix_rotation.ly"),
            |ast| interpret(ast),
            BatchSize::LargeInput,
        )
    });
    c.bench_function("tree", |b| {
        b.iter_batched(
            || ast!("../src/interpreter/tests/implementation/tree.ly"),
            |ast| interpret(ast),
            BatchSize::LargeInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
