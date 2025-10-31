# Multi-stage Dockerfile for The Agency
# Stage 1: Builder
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/bin/agency-daemon.rs && \
    echo "fn main() {}" > src/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release --bin agency-daemon && \
    rm -rf src target/release/deps/the_agency* target/release/deps/agency_daemon*

# Copy actual source code
COPY src ./src
COPY examples ./examples

# Build the actual application
RUN cargo build --release --bin agency-daemon

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 agency && \
    mkdir -p /etc/the-agency /var/lib/the-agency /var/log/the-agency && \
    chown -R agency:agency /etc/the-agency /var/lib/the-agency /var/log/the-agency

# Copy binary from builder
COPY --from=builder /app/target/release/agency-daemon /usr/local/bin/agency-daemon

# Copy default config
COPY config.example.toml /etc/the-agency/config.toml
RUN chown agency:agency /etc/the-agency/config.toml

# Switch to app user
USER agency
WORKDIR /var/lib/the-agency

# Expose API port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the daemon
ENTRYPOINT ["/usr/local/bin/agency-daemon"]
CMD ["--config", "/etc/the-agency/config.toml", "--host", "0.0.0.0", "--port", "8080"]
