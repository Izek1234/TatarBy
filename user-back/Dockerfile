FROM rust:1.88.0

WORKDIR /usr/src/app

# Install build deps + OpenSSL headers for sqlx/openssl-sys
RUN apt-get update && \
    apt-get install -y \
        libssl-dev \
        pkg-config \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy source + manifests
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY migrations/ ./migrations/

# 👇 SET DATABASE_URL AT BUILD TIME (required for SQLx macros)
# You MUST pass this when building, e.g.:
# docker build --build-arg DATABASE_URL=postgres://user:pass@host/db .

ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL

# Optional: verify DB is reachable before building (uncomment if needed)
# RUN cargo install sqlx-cli && sqlx db ping || echo "⚠️ DB not reachable, build may fail if query cache is outdated"

# Build and run in one step? No — we’ll run with `cargo run` later
# Just ensure it compiles
#RUN cargo build

# Create non-root user
RUN adduser --disabled-password --gecos '' appuser && chown -R appuser:appuser /usr/src/app
USER appuser

CMD ["cargo" "install" "sqlx-cli"]

CMD ["sqlx" "migrate" "run"]

CMD ["cargo" "sqlx" "prepare"]

EXPOSE 8000

# Run the app with cargo run (hot reload? no — but simple)
CMD ["cargo", "run"]
