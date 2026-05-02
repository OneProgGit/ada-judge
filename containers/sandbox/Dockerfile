FROM debian:trixie-slim as sandbox

# Install clang, go, rust & other required system packages
RUN apt-get update && apt-get install -y \
    curl build-essential ca-certificates clang lld git \
    xz-utils gcc g++ make libgmp-dev libffi-dev libncurses-dev libtinfo-dev \
    golang-go rustc \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m runner
RUN mkdir -p /sandbox && chown runner:runner /sandbox

USER runner
WORKDIR /sandbox
