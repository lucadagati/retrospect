# Multi-stage build for Wasmbed microROS Bridge
FROM rust:1.88-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build the application
RUN cargo build --release -p wasmbed-microros-bridge

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false wasmbed

# Copy the binary
COPY --from=builder /app/target/release/wasmbed-microros-bridge /usr/local/bin/

# Set ownership
RUN chown wasmbed:wasmbed /usr/local/bin/wasmbed-microros-bridge

# Switch to non-root user
USER wasmbed

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["wasmbed-microros-bridge"]
