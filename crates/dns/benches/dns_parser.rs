use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dns::parse::parser::DnsParser;

fn dns_parser(c: &mut Criterion) {
    let dns_queries: Vec<_> = (include_bytes!("./inputs/1000.bin").chunks(512))
        .chain(include_bytes!("./inputs/youtube-spotify.bin").chunks(512))
        .map(|chunk| {
            let mut packet = [0u8; 512];
            packet.copy_from_slice(chunk);
            packet
        })
        .collect();

    c.bench_function("parse full packet", |b| {
        b.iter(|| {
            for p in dns_queries.iter() {
                DnsParser::new(black_box(p)).parse().unwrap();
            }
        });
    });
}

criterion_group!(benches, dns_parser);
criterion_main!(benches);
