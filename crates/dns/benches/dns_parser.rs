use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dns::dns::DnsParser;

fn dns_parser(c: &mut Criterion) {
    let dns_queries: Vec<_> = (include_bytes!("./inputs/1000.bin").chunks(512))
        .chain(include_bytes!("./inputs/youtube-spotify.bin").chunks(512))
        .map(|c| c.to_vec())
        .collect();

    c.bench_function("answers parsing", |b| {
        b.iter(|| {
            for p in dns_queries.iter() {
                DnsParser::new(black_box(p)).parse_answers().unwrap();
            }
        });
    });

    c.bench_function("questions parsing", |b| {
        b.iter(|| {
            for p in dns_queries.iter() {
                DnsParser::new(black_box(p)).parse_question(1337);
            }
        });
    });
}

criterion_group!(benches, dns_parser);
criterion_main!(benches);
