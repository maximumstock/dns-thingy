FROM rust:1.68-buster as builder
WORKDIR /app
COPY . .
RUN CARGO_NET_GIT_FETCH_WITH_CLI=true CARGO_UNSTABLE_SPARSE_REGISTRY=true cargo build --release

FROM rust:1.68-slim-buster
WORKDIR /app
COPY --from=builder /app/target/release/dns-* /usr/local/bin
CMD ["dns-block-tokio"]
