# Tsukiyomi Bot - Discord Presence Tracker

A modern, production-ready Discord bot that tracks user presence changes and provides an HTTP API for querying presence data. Built with Rust using the latest package versions and future-proof design patterns.

## Features

- **Discord Presence Tracking**: Real-time tracking of user online status changes
- **PostgreSQL Database**: Persistent storage of presence data and history
- **Redis Caching**: Fast caching layer for improved performance
- **HTTP API**: RESTful API for querying presence data
- **In-Memory Storage**: DashMap for immediate access (like the original simple version)
- **Structured Logging**: Comprehensive logging with tracing
- **Production Ready**: Built for deployment on platforms like Render

## API Endpoints

- `GET /` - Health check endpoint
- `GET /presences` - Get all tracked presences
- `GET /presences/{user_id}` - Get specific user presence

## Environment Variables

```bash
# Required
DISCORD_TOKEN=your_discord_bot_token
DATABASE_URL=postgresql://user:password@localhost/dbname
PORT=8080

# Optional (with defaults)
REDIS_URL=redis://127.0.0.1:6379
LOG_LEVEL=info
REDIS_POOL_MAX_OPEN=16
REDIS_POOL_MAX_IDLE=8
DATABASE_MAX_CONNECTIONS=20
DATABASE_MIN_CONNECTIONS=5
```

## Quick Start

### 1. Setup Environment

Create a `.env` file:
```bash
cp env.sample .env
# Edit .env with your configuration
```

### 2. Database Setup

Run PostgreSQL migrations:
```bash
cargo install sqlx-cli
sqlx migrate run
```

### 3. Build and Run

```bash
# Development
cargo run

# Production
cargo build --release
./target/release/tsukiyomi-bot
```

## Package Versions (Latest)

- **Rust Edition**: 2021
- **Tokio**: 1.41+ (latest async runtime)
- **Serenity**: 0.12.4 (Discord API client)
- **SQLx**: 0.8.6 (Database toolkit)
- **Warp**: 0.3.7 (HTTP framework)
- **DashMap**: 6.1.0 (Concurrent HashMap)
- **Redis**: 0.23.3 (compatible with mobc-redis)
- **Tracing**: 0.1.41 (Structured logging)

## Architecture

The application follows a modular architecture:

```
src/
├── main.rs              # Application entry point
├── lib.rs               # Library exports
├── config/              # Configuration management
├── state/               # Application state
├── bot/                 # Discord bot handlers
├── api/                 # HTTP API routes and handlers
├── database/            # Database operations and models
├── cache/               # Redis caching layer
└── utils/               # Utility functions
```

## Deployment

### Docker

```dockerfile
FROM rust:1.82 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/tsukiyomi-bot /usr/local/bin/
CMD ["tsukiyomi-bot"]
```

### Render

The application is optimized for Render deployment:

1. **Web Service**: Automatically binds to `$PORT` and serves HTTP
2. **Background Worker**: Discord bot runs as background task
3. **Health Checks**: Built-in health endpoint at `/`
4. **Graceful Shutdown**: Proper cleanup on SIGTERM

## Development

### Prerequisites

- Rust 1.75+ (2021 edition)
- PostgreSQL 12+
- Redis 6+
- Discord Bot Token

### Running Tests

```bash
cargo test
```

### Code Quality

```bash
cargo clippy
cargo fmt
```

## Performance Features

- **Zero-Copy Serialization**: Efficient data handling
- **Connection Pooling**: Database and Redis connection pools
- **Async/Await**: Non-blocking I/O throughout
- **Structured Logging**: Low-overhead tracing

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

MIT License - see LICENSE file for details.
