FROM debian:trixie-slim AS sandbox

ENV PYTHONDONTWRITEBYTECODE=1
ENV PYTHONUNBUFFERED=1

# Install clang, go, rust & other required system packages
RUN apt-get update && apt-get install -y \
    curl build-essential ca-certificates clang lld git \
    xz-utils gcc g++ make libgmp-dev libffi-dev libncurses-dev libtinfo-dev \
    golang-go rustc python3 python3-pip python3-venv \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m runner
RUN mkdir -p /sandbox && chown runner:runner /sandbox

USER runner
WORKDIR /sandbox

RUN pip3 install --no-cache-dir --break-system-packages pyinstaller && \
    if [ -f requirements.txt ]; then pip3 install --no-cache-dir --break-system-packages -r requirements.txt; fi
