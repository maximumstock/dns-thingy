# dns-thingy

[![Continuous Integration](https://github.com/maximumstock/dns-thingy/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/maximumstock/dns-thingy/actions/workflows/rust.yml)

An educational project to learn more about the Domain Name System.
The goal is to build a minimal content filter system based on DNS similar to Adguard.

This workspace project consists of the following subcrates:

- `dns` - a library crate implementing DNS protocol specifics and a parser to consume DNS responses
- `dns-client` - a minimal DNS client that wraps `dns` to test resolving `A` and `CNAME` records for a given domain name
  and optionally given upstream DNS server (default `1.1.1.1`)
- `dns-block` - a single-threaded DNS server that selectively proxies queries to `1.1.1.1` or blocks blacklisted domains based on a list
- `dns-block-threaded` - a multi-threaded version of `dns-block`
- `dns-block-tokio` - an async version of `dns-block` based on Tokio (not fully async at this point, as it uses blocking parts of `dns`)

## todo

- [ ] parallelize dns-block
- [ ] cache records according to answer TTL
- [ ] implement more record types, ie. SOA

## Performance Evaluation

I'd like to use this opportunity to create a performance benchmark setup to get a better
feeling for performance characteristics of different implementation strategies, such as:

1. Single-threaded blocking (current implementation of `dns-block`)
2. Multi-threaded blocking
3. Multi-threaded async /w Tokio
4. ...

For benchmarking, I'll use [https://github.com/Tantalor93/dnspyre](https://github.com/Tantalor93/dnspyre).
Check out [benchmarks/run.sh](benchmarks/run.sh) for details.

