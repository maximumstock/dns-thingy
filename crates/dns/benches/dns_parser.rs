use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dns::dns::DnsParser;

fn dns_parser(c: &mut Criterion) {
    let dns_queries: Vec<_> = (include_bytes!("./inputs/1000.bin").chunks(512))
        .chain(include_bytes!("./inputs/youtube-spotify.bin").chunks(512))
        .map(|c| c.to_vec())
        .collect();

    c.bench_function("question parsing", |b| {
        let queries = dns_queries.clone();
        b.iter(|| {
            for p in queries.clone().into_iter() {
                DnsParser::new(black_box(p.try_into().unwrap())).parse_question();
            }
        });
    });

    c.bench_function("answers parsing", |b| {
        let queries = dns_queries.clone();
        b.iter(|| {
            for p in queries.clone().into_iter() {
                DnsParser::new(black_box(p.try_into().unwrap()))
                    .parse_answers()
                    .unwrap();
            }
        });
    });
}

criterion_group!(benches, dns_parser);
criterion_main!(benches);
