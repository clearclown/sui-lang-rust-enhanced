//! Benchmarks for the Sui interpreter

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sui_lang::Interpreter;

fn fibonacci_benchmark(c: &mut Criterion) {
    let code = r#"
# 0 1 {
< v0 a0 2
! v1 v0
? v1 1
^ a0
: 1
- v2 a0 1
$ v3 0 v2
- v4 a0 2
$ v5 0 v4
+ v6 v3 v5
^ v6
}
= g0 20
$ g1 0 g0
"#;

    c.bench_function("fibonacci(20)", |b| {
        b.iter(|| {
            let mut interp = Interpreter::new();
            interp.run(black_box(code), &[]).unwrap();
        })
    });
}

fn loop_benchmark(c: &mut Criterion) {
    let code = r#"
= v0 0
= v1 0
: 0
< v2 v0 1000
! v3 v2
? v3 1
+ v1 v1 v0
+ v0 v0 1
@ 0
: 1
"#;

    c.bench_function("loop_1000", |b| {
        b.iter(|| {
            let mut interp = Interpreter::new();
            interp.run(black_box(code), &[]).unwrap();
        })
    });
}

fn array_benchmark(c: &mut Criterion) {
    let code = r#"
[ v0 100
= v1 0
: 0
< v2 v1 100
! v3 v2
? v3 1
{ v0 v1 v1
+ v1 v1 1
@ 0
: 1
"#;

    c.bench_function("array_100", |b| {
        b.iter(|| {
            let mut interp = Interpreter::new();
            interp.run(black_box(code), &[]).unwrap();
        })
    });
}

fn simple_arithmetic_benchmark(c: &mut Criterion) {
    let code = r#"
= v0 1
= v1 2
+ v2 v0 v1
- v3 v2 v0
* v4 v3 v1
/ v5 v4 v1
% v6 v5 v0
"#;

    c.bench_function("simple_arithmetic", |b| {
        b.iter(|| {
            let mut interp = Interpreter::new();
            interp.run(black_box(code), &[]).unwrap();
        })
    });
}

criterion_group!(
    benches,
    fibonacci_benchmark,
    loop_benchmark,
    array_benchmark,
    simple_arithmetic_benchmark
);
criterion_main!(benches);
