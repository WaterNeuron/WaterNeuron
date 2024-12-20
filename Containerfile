# https://hub.docker.com/_/ubuntu
# Ubuntu Noble 24.04
FROM ubuntu@sha256:6e75a10070b0fcb0bead763c5118a369bc7cc30dfc1b0749c491bbb21f15c3c7
ENV TZ=UTC

# Install packages
RUN apt -y update && \
    apt -yqq install --no-install-recommends \
        curl \
        ca-certificates \
        git \
        g++ \
        && apt-get clean \
        && rm -rf /var/lib/apt/lists/*

RUN mkdir /waterneuron
WORKDIR /waterneuron

USER ubuntu

ENV PATH=/home/ubuntu/.cargo/bin:$PATH

ARG RUST_VERSION=1.83.0
RUN curl --fail https://sh.rustup.rs -sSf \
    | sh -s -- -y --default-toolchain ${RUST_VERSION}-x86_64-unknown-linux-gnu --no-modify-path && \
    rustup default ${RUST_VERSION}-x86_64-unknown-linux-gnu && \
    rustup target add wasm32-unknown-unknown && \
    rustup component add clippy

RUN cargo install ic-wasm --version 0.9.1

CMD ["/usr/bin/bash"]
