FROM debian:bookworm-slim

RUN useradd -m runner
USER runner
WORKDIR /sandbox