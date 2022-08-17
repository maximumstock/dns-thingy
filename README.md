# dns-thingy

[![Continuous Integration](https://github.com/maximumstock/dns-thingy/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/maximumstock/dns-thingy/actions/workflows/rust.yml)

An educational project to learn more about the Domain Name System.
The goal is to build a minimal content filter system based on DNS similar to Adguard.

This workspace project consists of the following subcrates:

- `dns` - a library crate implementing DNS protocol specifics and a parser to consume DNS responses
- `dns-client` - a minimal DNS client that wraps `dns` to test resolving `A` and `CNAME` records for a given domain name
  and optionally given upstream DNS server (default `1.1.1.1`)
- `dns-block` - a DNS server that selectively proxies queries to `1.1.1.1` or blocks blacklisted domains based on a list
