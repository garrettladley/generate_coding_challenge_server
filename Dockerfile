FROM lukemathwalker/cargo-chef:latest-rust-1.71.0 as chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef as planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin generate_coding_challenge_server

FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/generate_coding_challenge_server generate_coding_challenge_server
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./generate_coding_challenge_server"]
