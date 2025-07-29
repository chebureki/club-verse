FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app
RUN apt-get update && apt-get install -y gettext cmake pkg-config libssl-dev libclang-dev clang \
  && rm -rf /var/lib/apt/lists/*

FROM chef AS planner
COPY ./src ./src
#COPY ./.sqlx ./.sqlx
COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY ./src ./src
#COPY ./.sqlx ./.sqlx
COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
RUN cargo build --release --bin box-node

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y gettext \
  && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/box-node /usr/local/bin
#ENTRYPOINT ["/usr/local/bin/fmx_search_proxy", "-c", "/config.toml" ]
