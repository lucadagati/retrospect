FROM rust:1.88-slim as builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY resources/ ./resources/

RUN cargo build --release --bin wasmbed-gateway --bin wasmbed-gateway-simple

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/wasmbed-gateway /usr/local/bin/
COPY --from=builder /app/target/release/wasmbed-gateway-simple /usr/local/bin/
COPY --from=builder /app/resources/dev-certs /etc/wasmbed

EXPOSE 4423 8080

CMD ["wasmbed-gateway"]
