build-release:
	cargo build --release
benchmark:
	dnspyre -s "127.0.0.1:53000" -n 10 -c 8 -t A --recurse --no-color https://raw.githubusercontent.com/Tantalor93/dnspyre/master/data/2-domains
benchmark-all: build-release
	./benchmarks/run.sh
tidy:
	cargo fmt && cargo clippy -- -D warnings
build-docker:
	docker buildx build --no-cache -f Dockerfile -t dns-thingy:latest .
test:
	cargo test --all-features --all-targets -- --nocapture
dev:
	cargo watch -w crates -x "run --bin dns-block-tokio"
