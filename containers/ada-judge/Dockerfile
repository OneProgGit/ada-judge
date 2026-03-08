FROM rust:1.94.0 AS builder
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
COPY . .
RUN cargo build --release
FROM debian:trixie-slim
WORKDIR /usr/local/bin
COPY --from=builder /usr/src/app/target/release/ada-judge ./ada-judge
CMD ["./ada-judge"]
