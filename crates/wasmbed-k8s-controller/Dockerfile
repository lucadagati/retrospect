FROM debian:testing-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the pre-built binary
COPY target/release/wasmbed-k8s-controller /app/wasmbed-k8s-controller

EXPOSE 8080

CMD ["/app/wasmbed-k8s-controller"]
