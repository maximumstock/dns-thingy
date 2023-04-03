#!/usr/bin/env bash

# set -ex

DOMAINS_URL="https://raw.githubusercontent.com/Tantalor93/dnspyre/master/data/1000-domains"
DOMAINS_100=$(curl $DOMAINS_URL | head -n 100)

export PATH="./releases:./target/release:$PATH"

mkdir -p benchmarks/{basic,threaded-4,tokio}

DNS_BENCHMARK=true PORT=53000 dns-block &
echo "Started dns-block"
DNS_BENCHMARK=true PORT=53001 dns-block-threaded &
echo "Started dns-block-threaded"
DNS_BENCHMARK=true PORT=53002 dns-block-tokio &
echo "Started dns-block-tokio"

sleep 3

# dns-block
echo "Starting dns-block benchmark"
echo $DOMAINS_100 | xargs dnspyre -s "127.0.0.1:53000" -n 1 -c 3 -t A \
    --recurse \
    --distribution \
    --csv benchmarks/basic/raw.csv \
    --plot benchmarks/basic \
    --codes \
    > benchmarks/basic/stdout

# dns-block-threaded
echo "Starting dns-block-threaded benchmark"
echo $DOMAINS_100 | xargs dnspyre -s "127.0.0.1:53001" -n 1 -c 3 -t A \
    --recurse \
    --distribution \
    --csv benchmarks/threaded-4/raw.csv \
    --plot benchmarks/threaded-4 \
    --codes \
    > benchmarks/threaded-4/stdout

# dns-block-tokio
echo "Starting dns-block-tokio benchmark"
echo $DOMAINS_100 | xargs dnspyre -s "127.0.0.1:53002" -n 1 -c 3 -t A \
    --recurse \
    --distribution \
    --csv benchmarks/tokio/raw.csv \
    --plot benchmarks/tokio \
    --codes \
    > benchmarks/tokio/stdout

echo "Killing servers..."
pkill dns-block || true
pkill dns-block-threaded || true
pkill dns-block-tokio || true
