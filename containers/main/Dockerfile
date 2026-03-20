FROM rust:1.94.0 AS builder
WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main {}" > src/main.rs
COPY . .
RUN cargo build --release

FROM debian:trixie-slim
WORKDIR /usr/local/bin
RUN apt-get update && apt-get install -y \
    docker.io curl build-essential ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
COPY --from=builder /usr/src/app/target/release/ada-judge ./ada-judge
CMD ["./ada-judge"]
