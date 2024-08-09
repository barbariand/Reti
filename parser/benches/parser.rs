use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, Criterion};
use parser::ast::simplify::Simplify;
use parser::lexer::Lexer;
use parser::parse;
use parser::prelude::{Evaluator, MathContext};
use std::hint::black_box;
use tokio::runtime::Runtime;

fn parse_and_evaluation(c: &mut Criterion) {
    let rt = &tokio::runtime::Runtime::new().unwrap();
    let context = &MathContext::new();
    bench_all(c, rt, context, "1+1");
    bench_all(c, rt, context, "1+1+1");
    bench_all(c, rt, context, "1+1+1+1");
    bench_all(c, rt, context, "1+2+3+(4+5)*6");
}

criterion_group!(benches, parse_and_evaluation);
criterion_main!(benches);

//UTILS
fn bench_all(
    c: &mut Criterion,
    rt: &Runtime,
    context: &MathContext,
    input: &'static str,
) {
    let group = &mut c.benchmark_group(format!("benching input \"{}\"", input));
    run_parse_bench(group, rt, context, input);
    run_eval_bench(group, rt, context, input);
}

fn run_parse_bench(
    group: &mut BenchmarkGroup<WallTime>,
    rt: &Runtime,
    context: &MathContext,
    input: &'static str,
) {
    group.bench_function(&format!("parsing with input: \"{}\"", input), |b| {
        b.to_async(rt).iter(|| black_box(parse(input, &context)))
    });
}
fn run_eval_bench(
    group: &mut BenchmarkGroup<WallTime>,
    rt: &Runtime,
    context: &MathContext,
    input: &'static str,
) {
    let ast = rt
        .block_on(parse(input, context))
        .expect("could not evaluate ast for bench");
    group.bench_function(
        &format!("evaluating with input: \"{}\"", input),
        |b| b.iter(|| black_box(ast.clone().simple(context))),
    );
}
