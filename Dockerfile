# Multi-stage build for optimized production image
FROM rust:1.82 as builder

# Create app directory
WORKDIR /app

# Install required system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the application in release mode
RUN cargo build --release

# Runtime stage with minimal base image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user for security
RUN useradd -r -s /bin/false -m tsukiyomi

# Copy the binary from builder stage
COPY --from=builder /app/target/release/tsukiyomi-bot /usr/local/bin/tsukiyomi-bot

# Set ownership and permissions
RUN chown tsukiyomi:tsukiyomi /usr/local/bin/tsukiyomi-bot \
    && chmod +x /usr/local/bin/tsukiyomi-bot

# Switch to non-root user
USER tsukiyomi

# Expose the port (will be overridden by $PORT env var)
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:${PORT:-8080}/ || exit 1

# Set default environment
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Run the binary
CMD ["tsukiyomi-bot"]
