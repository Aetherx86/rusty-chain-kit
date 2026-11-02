# Multi-stage build for Rusty Chain Kit
FROM rust:1.85-bookworm AS builder

WORKDIR /usr/src/rusty-chain-kit

COPY Cargo.toml Cargo.lock ./
COPY .cargo ./.cargo
COPY src ./src
COPY templates ./templates
COPY assets ./assets

RUN cargo build --release --bin rusty-chain-kit --bin para

# Runtime container
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

COPY --from=builder /usr/src/rusty-chain-kit/target/release/rusty-chain-kit /usr/local/bin/rusty-chain-kit
COPY --from=builder /usr/src/rusty-chain-kit/target/release/para /usr/local/bin/para

EXPOSE 3333 8545 8645

ENTRYPOINT ["para"]
CMD ["--help"]
