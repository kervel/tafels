# Build the server binary
FROM rust:latest AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY shared/ shared/
COPY server/ server/
# Dummy client crate to satisfy workspace
RUN mkdir -p client/src && echo 'fn main() {}' > client/src/main.rs
COPY client/Cargo.toml client/Cargo.toml
RUN cargo build --release -p tafels-server
RUN rm -rf client/

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/tafels-server /usr/local/bin/tafels-server
# WASM client dist is built externally and copied in via skaffold/CI
# If dist/ exists at build context, include it; otherwise server runs without static files
COPY dist/ /app/dist/
ENV STATIC_DIR=/app/dist
EXPOSE 3000
CMD ["tafels-server"]
