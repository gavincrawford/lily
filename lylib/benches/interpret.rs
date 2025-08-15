use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use lylib::{
    interpreter::Interpreter,
    lexer::Lexer,
    parser::{ASTNode, Parser},
};
use std::{
    hint::black_box,
    io::{stdin, stdout},
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

fn interpret(ast: Rc<ASTNode>) {
    let mut i = Interpreter::new(stdin(), stdout());
    i.execute(ast).expect("failed to execute benchmark AST.");
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fibonacci", |b| {
        b.iter_batched(
            || ast!("../src/interpreter/tests/implementation/fibonacci.ly"),
            |ast| interpret(ast),
            BatchSize::SmallInput,
        )
    });
    c.bench_function("matrix rotation", |b| {
        b.iter_batched(
            || ast!("../src/interpreter/tests/implementation/matrix_rotation.ly"),
            |ast| interpret(ast),
            BatchSize::SmallInput,
        )
    });
    c.bench_function("tree", |b| {
        b.iter_batched(
            || ast!("../src/interpreter/tests/implementation/tree.ly"),
            |ast| interpret(ast),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
