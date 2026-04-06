# Multi-stage build for Indonesia Cybersecurity Incident Index
FROM rust:1.94-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r id_siber && useradd -r -g id_siber id_siber

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/idsiber /usr/local/bin/idsiber
COPY --from=builder /app/target/release/migrate /usr/local/bin/migrate

# Copy configuration files
COPY deny.toml ./

# Create directories
RUN mkdir -p /app/logs /app/uploads && \
    chown -R id_siber:id_siber /app

# Switch to non-root user
USER id_siber

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# Default command
CMD ["idsiber"]

# Development stage
FROM builder as development
WORKDIR /app
RUN cargo install cargo-watch
CMD ["cargo", "watch", "-x", "run", "--bin", "idsiber"]

# API stage (for docker-compose)
FROM runtime as api
CMD ["idsiber"]

# Migration stage (for docker-compose)
FROM runtime as migrate
ENTRYPOINT ["migrate"]
