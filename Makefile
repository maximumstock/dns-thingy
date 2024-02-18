build-release:
	cargo build --release
benchmark: build-release
	./benchmarks/run.sh
tidy:
	cargo fmt && cargo clippy -- -D warnings
build-docker:
	docker buildx build --no-cache -f Dockerfile -t dns-thingy:latest .
test:
	cargo test --all-features --all-targets -- --nocapture
