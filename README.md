# dns-thingy

An educational project to learn more about the DNS.
The goal is to build a minimal content filter system based on DNS similar to Adguard.

This workspace project consists of the following subcrates:

- `dns` - a library crate implementing DNS protocol specifics and a parser to consume DNS responses
- `dns-client` - a minimal DNS client that wraps `dns` to test resolving `A` and `CNAME` records for a given domain name
  and optionally given upstream DNS server (default `1.1.1.1`)
