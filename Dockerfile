FROM rust:1.74-buster as builder
WORKDIR /app
COPY . .
RUN CARGO_NET_GIT_FETCH_WITH_CLI=true CARGO_UNSTABLE_SPARSE_REGISTRY=true cargo build --release
RUN strip /app/target/release/dns-block-tokio

FROM rust:1.74-slim-buster
WORKDIR /app
COPY --from=builder /app/target/release/dns-block-tokio /usr/local/bin/dns-block-tokio
EXPOSE 53000/udp
CMD ["dns-block-tokio"]
