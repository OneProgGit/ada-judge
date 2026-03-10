FROM rust:1.94.0 AS builder
WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main {}" > src/main.rs
COPY . .
RUN cargo build --release

FROM debian:trixie-slim
WORKDIR /usr/local/bin
RUN apt-get update && apt-get install -y docker.io

COPY --from=builder /usr/src/app/target/release/ada-judge ./ada-judge
CMD ["./ada-judge"]
