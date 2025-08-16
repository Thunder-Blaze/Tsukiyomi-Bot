# --- Build Stage ---
FROM clux/muslrust:stable AS builder

WORKDIR /app
COPY . .

# Build the bot as a static binary
RUN cargo build --release

# --- Minimal Runtime Stage ---
FROM alpine:3.20

WORKDIR /app

# Install ca-certificates (for HTTPS), remove temp
RUN apk add --no-cache ca-certificates

# Copy the statically-built bot binary
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/tsukiyomi-bot /app/tsukiyomi-bot

EXPOSE 10000

ENV PORT=10000

CMD ["./tsukiyomi-bot"]
