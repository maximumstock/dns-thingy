#!/usr/bin/env bash

set -ex

DOMAINS_URL="https://raw.githubusercontent.com/Tantalor93/dnspyre/master/data/1000-domains"
DOMAINS_100=$(curl -s "$DOMAINS_URL" | head -n 100)

export PATH="./releases:./target/release:$PATH"

OUTPUT_PATH="benchmarks/results"
mkdir -p $OUTPUT_PATH

echo "Starting dns-block-tokio"
dns-block-tokio --benchmark --resolution-delay-ms 100 --bind-port 53000 --quiet &

sleep 3

if [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
    echo "Starting perf recording"
    sudo sysctl -w kernel.perf_event_paranoid=1
    sudo perf record -F99 -a -p $(pgrep dns-block-tokio) -o $OUTPUT_PATH/perf.data &
fi

echo "Starting dns-block-tokio benchmark"
echo $DOMAINS_100 | xargs dnspyre -s "127.0.0.1:53000" -n 1 -c 3 -t A \
    --recurse \
    --csv $OUTPUT_PATH/raw.csv \
    --plot $OUTPUT_PATH \
    --no-color \
    > $OUTPUT_PATH/stdout

echo "Killing servers..."
if [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
    pkill -SIGTERM perf || true

    sleep 5 # give perf time to write the output file

    git clone --depth 1 http://github.com/brendangregg/FlameGraph
    cd FlameGraph

    perf script -i ../$OUTPUT_PATH/perf.data | ./stackcollapse-perf.pl > out.perf-folded || true
    cat out.perf-folded | ./flamegraph.pl > perf-kernel.svg || true

    mv out.perf-folded ../$OUTPUT_PATH/out.perf-folded
    mv perf-kernel.svg ../$OUTPUT_PATH/perf-kernel.svg
fi

pkill -SIGTERM dns-block-tokio || true

# ./stackcollapse-perf.pl ../out.stacks1 > out.folded1
# ./stackcollapse-perf.pl ../out.stacks2 > out.folded2
# ./difffolded.pl out.folded1 out.folded2 | ./flamegraph.pl > diff2.svg
