use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dns::dns::DnsParser;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("question parsing", |b| {
        b.iter(|| DnsParser::new(black_box(vec![])).parse_question())
    });

    c.bench_function("answers parsing", |b| {
        b.iter(|| DnsParser::new(black_box(vec![])).parse_answers())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
