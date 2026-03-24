# based on glibc due to boring-sys2 dependency currently being broken on musl

FROM rust:slim-bookworm AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y \
    cmake \
    golang \
    perl \
    clang \
    libclang-dev \
    llvm \
    make \
    g++ \
    pkg-config \
    ca-certificates \
    git \
    && rm -rf /var/lib/apt/lists/*
COPY . .
ENV RUST_BACKTRACE=1
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12 AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/metasearch /usr/local/bin/metasearch
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
ARG CONFIG
ENV CONFIG=${CONFIG}
EXPOSE 28019
ENTRYPOINT ["/usr/local/bin/metasearch"]
