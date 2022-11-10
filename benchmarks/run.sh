#!/usr/bin/env bash
set -ex
mkdir -p benchmarks/{basic,threaded-4,tokio}

DOMAINS="https://raw.githubusercontent.com/Tantalor93/dnspyre/master/data/1000-domains"

echo "Building releases"
cargo build --release

PORT=53000 cargo run --release -p dns-block &
echo "Started dns-block"
PORT=53001 cargo run --release -p dns-block-threaded &
echo "Started dns-block-threaded"
PORT=53002 cargo run --release -p dns-block-tokio &
echo "Started dns-block-tokio"

# dns-block
dnspyre -s "127.0.0.1:53000" -n 1 -t A \
    --distribution \
    --csv benchmarks/basic.csv \
    --plot benchmarks/basic \
    --codes $DOMAINS \
    > benchmarks/basic/stdout

# dns-block-threaded
dnspyre -s "127.0.0.1:53001" -n 1 -t A \
    --distribution \
    --csv benchmarks/threaded-4.csv \
    --plot benchmarks/threaded-4 \
    --codes $DOMAINS \
    > benchmarks/threaded-4/stdout

# dns-block-tokio
dnspyre -s "127.0.0.1:53002" -n 1 -t A \
    --distribution \
    --csv benchmarks/tokio.csv \
    --plot benchmarks/tokio \
    --codes $DOMAINS \
    > benchmarks/tokio/stdout

echo "Killing servers..."
pkill dns-block
pkill dns-block-threaded
pkill dns-block-tokio
