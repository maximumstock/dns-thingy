build-release:
	cargo build --release
benchmark: build-release
	./benchmarks/run.sh
tidy:
	cargo fmt && cargo clippy -- -D warnings
