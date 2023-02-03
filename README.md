# dns-thingy

[![Continuous Integration](https://github.com/maximumstock/dns-thingy/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/maximumstock/dns-thingy/actions/workflows/rust.yml)

A minimal work-in-progress content filter system based on DNS similar to Adguard.
This project serves the purpose to learn more about the [Domain Name System](https://en.wikipedia.org/wiki/Domain_Name_System).

This workspace project consists of the following subcrates in `crates`:

- `dns` - a library crate implementing DNS protocol specifics and a parser to consume DNS responses
- `dns-client` - a minimal DNS client that wraps `dns` to test resolving `A` and `CNAME` records for a given domain name
  and optionally given upstream DNS server (default `1.1.1.1`)
- `dns-block` - a single-threaded DNS server that selectively proxies queries to `1.1.1.1` or blocks blacklisted domains based on a list
- `dns-block-threaded` - a multi-threaded version of `dns-block`
- `dns-block-tokio` - an async version of `dns-block` based on Tokio (not fully async at this point, as it uses blocking parts of `dns`)

## TODO

- [ ] fix: find query that results in. looks like it happens consistently across server implementations for a specific query.

```
thread '<unnamed>' panicked at 'range start index 506 out of range for slice of length 358', dns/src/dns.rs:101:9
stack backtrace:
   0: rust_begin_unwind
             at /rustc/897e37553bba8b42751c67658967889d11ecd120/library/std/src/panicking.rs:584:5
   1: core::panicking::panic_fmt
             at /rustc/897e37553bba8b42751c67658967889d11ecd120/library/core/src/panicking.rs:142:14
   2: core::slice::index::slice_start_index_len_fail_rt
             at /rustc/897e37553bba8b42751c67658967889d11ecd120/library/core/src/slice/index.rs:53:5
   3: core::slice::index::slice_start_index_len_fail
             at /rustc/897e37553bba8b42751c67658967889d11ecd120/library/core/src/slice/index.rs:42:9
   4: dns::dns::DnsParser::parse_domain_name_rec
   5: dns::dns::DnsParser::parse_answer
   6: <alloc::vec::Vec<T> as alloc::vec::spec_from_iter::SpecFromIter<T,I>>::from_iter
   7: dns::resolver::resolve
```

- [ ] fix: cant pipe through dns query as we need to recursively query upstream but downstream might not have set that flag
- [ ] add custom blocking rules
- [ ] parallelize dns-block
- [ ] cache records according to answer TTL
- [ ] implement more record types

## Performance Evaluation

I'd like to use this opportunity to create a performance benchmark setup to get a better
feeling for performance characteristics of different implementation strategies, such as:

1. Single-threaded blocking (current implementation of `dns-block`)
2. Multi-threaded blocking
3. Asynchronous based on Tokio

See [Benchmarks](benchmarks/README.md) for further information.
