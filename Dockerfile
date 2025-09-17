# --- Build Stage ---
FROM clux/muslrust:stable AS builder

# Set build environment for faster builds
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Pre-compile dependencies (this layer will be cached unless dependencies change)
RUN cargo build --release && rm -rf src target/x86_64-unknown-linux-musl/release/deps/tsukiyomi*

# Copy source code and migrations
COPY src ./src
COPY migrations ./migrations

# Build the actual application with optimizations
RUN cargo build --release --locked

# --- Minimal Runtime Stage ---
FROM alpine:3.20

# Install only essential runtime dependencies
RUN apk add --no-cache ca-certificates tzdata && \
    addgroup -g 1000 appuser && \
    adduser -D -s /bin/sh -u 1000 -G appuser appuser && \
    rm -rf /var/cache/apk/*

WORKDIR /app

# Copy the statically-built bot binary with correct ownership
COPY --from=builder --chown=appuser:appuser /app/target/x86_64-unknown-linux-musl/release/tsukiyomi-bot /app/tsukiyomi-bot

# Make binary executable
RUN chmod +x /app/tsukiyomi-bot

# Switch to non-root user for security
USER appuser

EXPOSE 10000

ENV PORT=10000

# ENV RUST_LOG=info
# ENV RUST_BACKTRACE=1

CMD ["./tsukiyomi-bot"]
