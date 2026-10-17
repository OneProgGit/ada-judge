FROM lukemathwalker/cargo-chef:latest-rust-1.99.1-alpine3.24 AS chef
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

FROM chef AS builder-tester
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY --from=cacher /usr/src/app/target /usr/src/app/target
COPY . .
RUN cargo build --release -p tester

FROM debian:trixie-slim AS main
WORKDIR /usr/local/bin

# Install docker & other required system packages
RUN apt-get update && apt-get install -y \
    docker.io curl build-essential ca-certificates lld git \
    xz-utils gcc g++ make libgmp-dev libffi-dev libncurses-dev libtinfo-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder-main /usr/src/app/target/release/ada-judge ./ada-judge

ARG DOCKER_GID

RUN groupadd --gid 1000 runner \
    && useradd \
        --uid 1000 \
        --gid 1000 \
        --create-home \
        --shell /bin/bash runner

USER 1000:1000

CMD ["./ada-judge"]

FROM debian:trixie-slim AS tester
WORKDIR /usr/local/bin

# Install docker & other required system packages
RUN apt-get update && apt-get install -y \
    docker.io curl build-essential ca-certificates lld git \
    xz-utils gcc g++ make libgmp-dev libffi-dev libncurses-dev libtinfo-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder-tester /usr/src/app/target/release/tester ./tester

ARG DOCKER_GID

RUN groupadd --gid 1000 runner \
    && useradd \
        --uid 1000 \
        --gid 1000 \
        --create-home \
        --shell /bin/bash runner

USER 1000:1000

CMD ["./tester"]
