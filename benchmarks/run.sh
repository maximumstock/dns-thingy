#!/usr/bin/env bash

set -e

DOMAINS_URL="https://raw.githubusercontent.com/Tantalor93/dnspyre/master/data/1000-domains"
DOMAINS_100=$(curl -s "$DOMAINS_URL" | head -n 100)

export PATH="./releases:./target/release:$PATH"

mkdir -p benchmarks

dns-block-tokio --benchmark --resolution-delay-ms 10 --bind-port 53000 --quiet &
echo "Started dns-block-tokio"

sleep 3

# dns-block-tokio
echo "Starting dns-block-tokio benchmark"
echo $DOMAINS_100 | xargs dnspyre -s "127.0.0.1:53000" -n 1 -c 3 -t A \
    --recurse \
    --csv benchmarksraw.csv \
    --plot benchmarks \
    --no-color \
    > benchmarks/stdout

echo "Killing servers..."
pkill dns-block-tokio || true
