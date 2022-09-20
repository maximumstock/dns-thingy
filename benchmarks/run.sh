#!/usr/bin/env bash

# dns-block
dnspyre -s "127.0.0.1:53000" -n 1 -t A --distribution --csv benchmarks/basic-local-release.csv --codes "https://raw.githubusercontent.com/Tantalor93/dnspyre/master/data/1000-domains" > benchmarks/basic-local-release

# dns-block-threaded
dnspyre -s "127.0.0.1:53001" -n 1 -t A --distribution --csv benchmarks/threaded-4-local-release.csv --codes "https://raw.githubusercontent.com/Tantalor93/dnspyre/master/data/1000-domains" > benchmarks/threaded-4-local-release

# dns-block-tokio
dnspyre -s "127.0.0.1:53002" -n 1 -t A --distribution --csv benchmarks/tokio-local-release.csv --codes "https://raw.githubusercontent.com/Tantalor93/dnspyre/master/data/1000-domains" > benchmarks/tokio-local-release
