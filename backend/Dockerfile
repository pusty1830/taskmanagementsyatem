FROM rust:1-slim-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Dependency layer: rebuilt only when Cargo.toml / Cargo.lock change.
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin task-api

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
RUN useradd -r -u 10001 app
COPY --from=builder /app/target/release/task-api /usr/local/bin/task-api
USER app
EXPOSE 8080
CMD ["task-api"]
