build-release:
	cargo build --release
benchmark: build-release
	PATH="$$PATH:$$PWD/target/release" ./benchmarks/run.sh