# dns-thingy

[![Continuous Integration](https://github.com/maximumstock/dns-thingy/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/maximumstock/dns-thingy/actions/workflows/rust.yml)

An educational project to learn more about the Domain Name System.
The goal is to build a minimal content filter system based on DNS similar to Adguard.

This workspace project consists of the following subcrates:

- `dns` - a library crate implementing DNS protocol specifics and a parser to consume DNS responses
- `dns-client` - a minimal DNS client that wraps `dns` to test resolving `A` and `CNAME` records for a given domain name
  and optionally given upstream DNS server (default `1.1.1.1`)
- `dns-block` - a DNS server that selectively proxies queries to `1.1.1.1` or blocks blacklisted domains based on a list

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

`dnspyre -s "127.0.0.1:53000" -n 1 -c 2 -t A --distribution --csv out.csv --codes "https://raw.githubusercontent.com/Tantalor93/dnspyre/master/data/1000-domains"`

`dnspyre -s "127.0.0.1:53000" -n 100 -t A --distribution --csv benchmarks/basic-local-release.csv --codes "https://raw.githubusercontent.com/Tantalor93/dnspyre/master/data/2-domains" > benchmarks/basic-local-release`

Need a script that runs benchmarks and records the data for the current commit hash for each implementation. 