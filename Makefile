build-release:
	cargo build --release
benchmark-fishtank:
	dnspyre -s "192.168.0.113:53" -c 4 -t A --recurse --no-color https://raw.githubusercontent.com/Tantalor93/dnspyre/master/data/1000-domains
benchmark-nixpi:
	dnspyre -s "192.168.0.121:53" -c 4 -t A --recurse --no-color https://raw.githubusercontent.com/Tantalor93/dnspyre/master/data/1000-domains
benchmark-local:
	dnspyre -s "127.0.0.1:53000" -c 4 -t A --recurse --no-color https://raw.githubusercontent.com/Tantalor93/dnspyre/master/data/1000-domains
benchmark-all: build-release
	./benchmarks/run.sh
tidy:
	cargo fmt && cargo clippy -- -D warnings
build-docker:
	docker buildx build --no-cache -f Dockerfile -t dns-thingy:latest .
test:
	cargo test --all-features --all-targets -- --nocapture
dev:
	RUST_BACKTRACE=1 cargo watch -w crates -x "run --bin dns-block-tokio -- --caching-enabled"
deploy-nixpi:
	# Based on https://sebi.io/posts/2024-05-02-guide-cross-compiling-rust-from-macos-to-raspberry-pi-2024-apple-silicon/
	cargo build --release -p dns-block-tokio --target=armv7-unknown-linux-musleabihf
	rsync ./target/armv7-unknown-linux-musleabihf/release/dns-block-tokio nixpi:/root/.cargo/bin/dns-block-tokio
	ssh nixpi "systemctl restart dns-thingy"
logs-nixpi:
	ssh nixpi "journalctl -u dns-thingy -f"
deploy-fishtank:
	# Based on https://betterprogramming.pub/cross-compiling-rust-from-mac-to-linux-7fad5a454ab1
	cargo build --release -p dns-block-tokio --target=x86_64-unknown-linux-musl
	rsync ./target/x86_64-unknown-linux-musl/release/dns-block-tokio fishtank:/root/.cargo/bin/dns-block-tokio
	ssh fishtank "systemctl restart dns-thingy"
logs-fishtank:
	ssh fishtank "journalctl -u dns-thingy -f"
