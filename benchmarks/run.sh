#!/usr/bin/env bash

set -e

DOMAINS_URL="https://raw.githubusercontent.com/Tantalor93/dnspyre/master/data/1000-domains"
DOMAINS_100=$(curl -s "$DOMAINS_URL" | head -n 100)

export PATH="./releases:./target/release:$PATH"

mkdir -p benchmarks/tokio

dns-block-tokio --benchmark --port 53002 --quiet &
echo "Started dns-block-tokio"

sleep 3

# dns-block-tokio
echo "Starting dns-block-tokio benchmark"
echo $DOMAINS_100 | xargs dnspyre -s "127.0.0.1:53002" -n 1 -c 3 -t A \
    --recurse \
    --csv benchmarks/tokio/raw.csv \
    --plot benchmarks/tokio \
    > benchmarks/tokio/stdout

echo "Killing servers..."
pkill dns-block-tokio || true
