# Benchmarks

## Overview

We want to be able to benchmark our server implementations to compare implementation techniques and find performance regressions.

For now, there is a simple benchmark setup in `run.sh`, which should be run via `make benchmark` from the project root.
It depends solely on [dnspyre](https://github.com/Tantalor93/dnspyre).

## Benchmark Results

For each implementation the benchmark collects the following artefacts:

- DNS query response times histograms in `raw.csv`
- the `stdout` stream from the benchmarking tool `dnspyre`
- latency and throughput plots in `graphs-<timestamp>`

These results are not qualitative benchmarks and are not run in hygenic benchmarking environments, but they should give a basic stepping stone

## Continuous Integration

Each commit on `main` triggers a `benchmark` step after successful compilation & linting.
The benchmark artefacts are zipped and attached to the respective CI jobs for inspection.

The goal is to automatically embed these artefacts within a webpage to automatically detect performance regressions.
