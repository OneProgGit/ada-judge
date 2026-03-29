FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /usr/src/app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS cacher 
COPY --from=planner /usr/src/app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM chef AS builder-main
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY --from=cacher /usr/src/app/target /usr/src/app/target
COPY . .
RUN cargo build --release -p ada-judge

FROM chef AS builder-submissions-judger
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY --from=cacher /usr/src/app/target /usr/src/app/target
COPY . .
RUN cargo build --release -p submissions-judger

FROM debian:trixie-slim AS main
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

COPY --from=builder-main /usr/src/app/target/release/ada-judge ./ada-judge
CMD ["./ada-judge"]

FROM debian:trixie-slim AS submissions-judger
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

COPY --from=builder-submissions-judger /usr/src/app/target/release/submissions-judger ./submissions-judger
CMD ["./submissions-judger"]
