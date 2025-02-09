# dns-thingy

[![Continuous Integration](https://github.com/maximumstock/dns-thingy/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/maximumstock/dns-thingy/actions/workflows/rust.yml)

A minimal work-in-progress content filter system based on DNS similar to [AdguardHome](https://github.com/AdguardTeam/AdGuardHome).

This project serves the purpose to learn more about the [Domain Name System](https://en.wikipedia.org/wiki/Domain_Name_System).

This workspace project consists of the following subcrates in `crates`:

- `dns` - a library crate for constructing and consuming DNS packets (currently only supports DNS over UDP)
- `dns-client` - a minimal DNS client that wraps `dns` to test resolving `A` and `CNAME` records for a given domain name
  and optionally given upstream DNS server (default `1.1.1.1`)
- `dns-block-tokio` - an async stub resolver based on Tokio

## How to run

You can either clone and `cargo install -p dns-block-tokio` and then run `dns-block-tokio`
or use Docker (only `linux/amd64` images are being built by CI at the moment) by pulling the image
`docker run -p 53000:53000 maximumstock2/dns-thingy:latest`
which runs `dns-block-tokio` inside the container on port 53000 and makes it available on your host machine
on port `53000` as well. Optionally, use `-p 53:53000` to map your local port `53` to be used, but that most likely requires root privileges.

At that point `dns-block-tokio` can answer DNS queries, ie. `dig google.com @127.0.0.1 @53000`.

### Caching

You can optionally enable caching by passing `--caching-enabled` when running `dns-block-tokio`.
This will cache the DNS responses from your configured upstream relay based on the lowest time-to-live
values across all resource records in the DNS reponse.

## TODO

- [ ] feat: add custom blocking rules
- [ ] feat: add custom DNS rewrite rules
- [ ] api: request builder for DNS queries & responses
- [ ] bench
  - every commit on `master` should trigger a benchmark suite that collects the typical benchmark data, posts the data to the repository/GH Pages and builds a website with the results in a graph

## Request Flow

Syntax: https://mermaid.js.org/syntax/sequenceDiagram.html

```mermaid
sequenceDiagram
    participant user as User
    participant forwarder as dns-thingy
    participant relay as DNS Relay

    %% flow with filtering
    par flow with filtering
      user->>forwarder: dig A google.com
      activate forwarder
      forwarder->>forwarder: parse question, filter triggers
      forwarder->>forwarder: create NXDOMAIN answer with user's question ID
      forwarder->>user: return answer
      deactivate forwarder
    end

    %% flow without filtering
    par flow without filtering
      user->>forwarder: dig A google.de
      activate forwarder
      forwarder->>forwarder: parse question, filter does not trigger
      forwarder->>relay: forward original DNS question
      deactivate forwarder
      activate relay
      relay->>forwarder: DNS answer
      deactivate relay
      activate forwarder
      forwarder->>forwarder: parse answer, caching based on TTL
      forwarder->>user: return original DNS answer
      deactivate forwarder
    end
```

## References

Some reading material that was helpful:

- https://datatracker.ietf.org/doc/html/rfc1035
- https://github.com/EmilHernvall/dnsguide/blob/master/chapter1.md
- https://blog.cloudflare.com/how-to-stop-running-out-of-ephemeral-ports-and-start-to-love-long-lived-connections/
