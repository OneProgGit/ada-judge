FROM debian:trixie-slim as sandbox

# Install clang & other required system packages
RUN apt-get update && apt-get install -y \
    curl build-essential ca-certificates clang lld git \
    xz-utils gcc g++ make libgmp-dev libffi-dev libncurses-dev libtinfo-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Go
RUN curl -LO https://go.dev/dl/go1.26.2.linux-amd64.tar.gz \
    && tar -C /usr/local -xzf go1.26.2.linux-amd64.tar.gz \
    && rm go1.26.2.linux-amd64.tar.gz
ENV PATH="/usr/local/go/bin:${PATH}"

RUN useradd -m runner
RUN mkdir -p /sandbox && chown runner:runner /sandbox

USER runner
WORKDIR /sandbox

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
ENV PATH="/home/runner/.cargo/bin:${PATH}"

WORKDIR /sandbox
