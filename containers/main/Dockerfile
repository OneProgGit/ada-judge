FROM rust:1.94.1 AS builder
WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main {}" > src/main.rs
COPY . .
RUN cargo build --release

FROM debian:trixie-slim
WORKDIR /usr/local/bin

# Install docker & clang
RUN apt-get update && apt-get install -y \
    docker.io curl build-essential ca-certificates clang lld git \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Go
RUN curl -LO https://go.dev/dl/go1.26.1.linux-amd64.tar.gz \
    && tar -C /usr/local -xzf go1.26.1.linux-amd64.tar.gz \
    && rm go1.26.1.linux-amd64.tar.gz
ENV PATH="/usr/local/go/bin:${PATH}"

COPY --from=builder /usr/src/app/target/release/ada-judge ./ada-judge
CMD ["./ada-judge"]
