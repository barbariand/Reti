use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, Criterion};
use parser::ast::simplify::Simplify;
use parser::parse;
use parser::prelude::MathContext;
use std::hint::black_box;

fn parse_and_evaluation(c: &mut Criterion) {
    let context = &MathContext::new();
    bench_all(c, context, "1+1");
    bench_all(c, context, "1+1+1");
    bench_all(c, context, "1+1+1+1");
    bench_all(c, context, "1+2+3+(4+5)*6");
}

criterion_group!(benches, parse_and_evaluation);
criterion_main!(benches);

//UTILS
fn bench_all(c: &mut Criterion, context: &MathContext, input: &'static str) {
    let group = &mut c.benchmark_group(format!("benching input \"{}\"", input));
    run_parse_bench(group, context, input);
    run_eval_bench(group, context, input);
}

fn run_parse_bench(
    group: &mut BenchmarkGroup<WallTime>,
    context: &MathContext,
    input: &'static str,
) {
    group.bench_function(&format!("parsing with input: \"{}\"", input), |b| {
        b.iter(|| black_box(parse(input, &context)))
    });
}
fn run_eval_bench(
    group: &mut BenchmarkGroup<WallTime>,
    context: &MathContext,
    input: &'static str,
) {
    let ast = parse(input, context).expect("could not evaluate ast for bench");
    group.bench_function(
        &format!("evaluating with input: \"{}\"", input),
        |b| b.iter(|| black_box(ast.clone().simple(context))),
    );
}
