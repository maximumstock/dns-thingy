#!/usr/bin/env bash

set -e

DOMAINS_URL="https://raw.githubusercontent.com/Tantalor93/dnspyre/master/data/1000-domains"
DOMAINS_100=$(curl -s "$DOMAINS_URL" | head -n 100)

export PATH="./releases:./target/release:$PATH"

OUTPUT_PATH="benchmarks/results"
mkdir -p $OUTPUT_PATH

echo "Starting dns-block-tokio"
dns-block-tokio --benchmark --resolution-delay-ms 10 --bind-port 53000 --quiet &

sleep 3

echo "Starting perf recording"
if [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
    sudo sysctl -w kernel.perf_event_paranoid=1
fi
perf record -F99 --call-graph dwarf -p $(pgrep dns-block-tokio) -o $OUTPUT_PATH/perf.data &
ls -lah
tree $OUTPUT_PATH

echo "Starting dns-block-tokio benchmark"
echo $DOMAINS_100 | xargs dnspyre -s "127.0.0.1:53000" -n 1 -c 3 -t A \
    --recurse \
    --csv $OUTPUT_PATH/raw.csv \
    --plot $OUTPUT_PATH \
    --no-color \
    > $OUTPUT_PATH/stdout

echo "Killing servers..."
pkill -SIGTERM perf || true
pkill -SIGTERM dns-block-tokio || true
