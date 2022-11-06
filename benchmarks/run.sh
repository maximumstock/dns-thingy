#!/usr/bin/env bash

mkdir -p benchmarks/{basic-local,threaded-4-local,tokio-local}

DOMAINS="https://raw.githubusercontent.com/Tantalor93/dnspyre/master/data/1000-domains"

cargo build --release

# dns-block
cargo run --release -p dns-block &
sleep 2
dnspyre -s "127.0.0.1:53000" -n 1 -t A \
    --distribution \
    --csv benchmarks/basic-local-release.csv \
    --plot benchmarks/basic-local \
    --codes $DOMAINS \
    > benchmarks/basic-local/basic-local-release
pkill dns-block
sleep 2

# dns-block-threaded
PORT=53001 cargo run --release -p dns-block-threaded &
sleep 2
dnspyre -s "127.0.0.1:53001" -n 1 -t A \
    --distribution \
    --csv benchmarks/threaded-4-local-release.csv \
    --plot benchmarks/threaded-4-local \
    --codes $DOMAINS \
    > benchmarks/threaded-4-local/threaded-4-local-release
pkill dns-block-threaded
sleep 2

# dns-block-tokio
PORT=53002 cargo run --release -p dns-block-tokio &
sleep 2
dnspyre -s "127.0.0.1:53002" -n 1 -t A \
    --distribution \
    --csv benchmarks/tokio-local-release.csv \
    --plot benchmarks/tokio-local \
    --codes $DOMAINS \
    > benchmarks/tokio-local/tokio-local-release
pkill dns-block-tokio
