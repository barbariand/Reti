use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, Criterion};
use parser::ast::simplify::Simplify;
use parser::lexer::Lexer;
use parser::parse;
use parser::prelude::{Evaluator, MathContext};
use std::hint::black_box;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::channel;

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
fn parser(c: &mut Criterion) {
    let mut simple_group = c.benchmark_group("simple_parsing");
    let rt = &tokio::runtime::Runtime::new().unwrap();
    let context = &MathContext::new();
    run_parse_bench(&mut simple_group, rt, context, "1+1");
    run_parse_bench(&mut simple_group, rt, context, "1+1+1");
    run_parse_bench(&mut simple_group, rt, context, "1+1+1+1");
}

fn evaluation(c: &mut Criterion) {
    let mut simple_group = c.benchmark_group("simple_evaluation");

    let rt = &tokio::runtime::Runtime::new().unwrap();
    let context = &MathContext::new();

    run_eval_bench(&mut simple_group, rt, context, "1+1");
    run_eval_bench(&mut simple_group, rt, context, "1+1+1");
    run_eval_bench(&mut simple_group, rt, context, "1+1+1+1");
}
criterion_group!(benches, parser, evaluation);
criterion_main!(benches);
