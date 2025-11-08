FROM lukemathwalker/cargo-chef:latest-rust-1.91-alpine as chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM alpine:latest AS runtime
WORKDIR /app
COPY --from=builder /app/config.toml /usr/local/bin/config.toml
COPY --from=builder /app/target/release/metasearch /usr/local/bin/metasearch
ARG CONFIG
ENV CONFIG=${CONFIG}
EXPOSE 28019
ENTRYPOINT /usr/local/bin/metasearch $CONFIG
