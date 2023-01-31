build-release:
	cargo build --release
benchmark: build-release
	./benchmarks/run.sh