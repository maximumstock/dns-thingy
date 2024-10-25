#!/usr/bin/env bash

set -e

DOMAINS_URL="https://raw.githubusercontent.com/Tantalor93/dnspyre/master/data/1000-domains"
DOMAINS_100=$(curl -s "$DOMAINS_URL" | head -n 100)

export PATH="./releases:./target/release:$PATH"

OUTPUT_PATH="benchmarks/results"
mkdir -p $OUTPUT_PATH

# attach cargo-flamegraph to the running server process
flamegraph \
    -o $OUTPUT_PATH/flamegraph.svg \
    --deterministic \
    dns-block-tokio -- --benchmark --resolution-delay-ms 10 --bind-port 53000 --quiet &
echo "Started dns-block-tokio"

sleep 3

# dns-block-tokio
echo "Starting dns-block-tokio benchmark"
echo $DOMAINS_100 | xargs dnspyre -s "127.0.0.1:53000" -n 1 -c 3 -t A \
    --recurse \
    --csv $OUTPUT_PATH/raw.csv \
    --plot $OUTPUT_PATH \
    --no-color \
    > $OUTPUT_PATH/stdout

echo "Killing servers..."
pkill dns-block-tokio || true
pkill flamegraph || true
