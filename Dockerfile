# https://hub.docker.com/_/ubuntu
# 24.10 - oracular-20240617
FROM ubuntu@sha256:2a1e42397521001f21178a06e37ba1024481d3e8b6a754902ac5fb6a0861c7ac
ENV TZ=UTC

# TODO download every deb dependencies and rust binary

# Install packages
RUN apt -yq update && \
    apt -yqq install \
        curl \
        git \
        gcc \
        lld \
        sudo \
        wget \
        tree \
        cmake \
        wabt \
        build-essential \
        pkg-config \
        libssl-dev \
        libunwind-dev \
        libusb-1.0-0-dev \
        libsqlite3-dev \
        zlib1g-dev \
        libclang-18-dev \
        protobuf-compiler \
        llvm \
        liblmdb-dev \
        liblzma-dev \
        && apt-get clean \
        && rm -rf /var/lib/apt/lists/*

# No password sudo
RUN echo "ubuntu ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

RUN mkdir /waterneuron

USER ubuntu

# Set PATH
ENV PATH=/home/ubuntu/.cargo/bin:/home/ubuntu/.local/bin:$PATH

# Add Rust/Cargo support
ARG RUST_VERSION=1.79.0
RUN curl --fail https://sh.rustup.rs -sSf \
    | sh -s -- -y --default-toolchain ${RUST_VERSION}-x86_64-unknown-linux-gnu --no-modify-path && \
    rustup default ${RUST_VERSION}-x86_64-unknown-linux-gnu && \
    rustup target add wasm32-unknown-unknown && \
    rustup component add clippy

# Install ic-wasm -- TODO instead fetch and pin the dependency https://github.com/dfinity/ic-wasm/releases/tag/0.9.1
RUN cargo install ic-wasm

CMD ["/usr/bin/bash"]
