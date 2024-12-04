FROM rust:latest AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y protobuf-compiler
RUN rustup target add x86_64-unknown-linux-musl

COPY server/Cargo.toml .
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl

COPY proto ../proto
COPY server/src src
COPY server/build.rs .
RUN cargo build --release --target x86_64-unknown-linux-musl

RUN strip target/x86_64-unknown-linux-musl/release/dsqs-server

FROM alpine:latest
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/dsqs-server .

CMD ["./dsqs-server"]