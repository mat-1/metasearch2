FROM lukemathwalker/cargo-chef:latest-rust-alpine as chef
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
RUN apk add sed
RUN sed -i 's/\[::\]/0.0.0.0/g' /app/src/web/mod.rs
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY . .
RUN apk add sed
RUN sed -i 's/\[::\]/0.0.0.0/g' /app/src/web/mod.rs
RUN cargo build --release
RUN mv ./target/release/metasearch2 ./app

FROM scratch AS runtime
WORKDIR /app
COPY --from=builder /app/app /usr/local/bin/
EXPOSE 28019
ENTRYPOINT ["/usr/local/bin/app"]
