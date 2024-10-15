# https://hub.docker.com/_/ubuntu
# 24.10 - oracular-20240617
FROM ubuntu@sha256:2a1e42397521001f21178a06e37ba1024481d3e8b6a754902ac5fb6a0861c7ac
ENV TZ=UTC

# Install packages
RUN apt -yq update && \
    apt -yqq install \
        curl \
        git \
        vim \
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

# Add bazel
ARG bazelisk_sha=d28b588ac0916abd6bf02defb5433f6eddf7cba35ffa808eabb65a44aab226f7
RUN curl -fsSL https://github.com/bazelbuild/bazelisk/releases/download/v1.19.0/bazelisk-linux-amd64 -o /usr/bin/bazel && \
    echo "$bazelisk_sha /usr/bin/bazel" | sha256sum --check && \
    chmod 777 /usr/bin/bazel

# Add buildifier
ARG buildifier_sha=be63db12899f48600bad94051123b1fd7b5251e7661b9168582ce52396132e92
RUN curl -fsSL https://github.com/bazelbuild/buildtools/releases/download/v6.4.0/buildifier-linux-amd64 -o /usr/bin/buildifier && \
    echo "$buildifier_sha /usr/bin/buildifier" | sha256sum --check && \
    chmod 777 /usr/bin/buildifier

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

# Install ripgrep
RUN cargo install ripgrep ic-wasm tokei git-delta bat

# Copy .vimrc
COPY --chown=ubuntu:ubuntu scripts/data/.vimrc /home/ubuntu/.vimrc

# Install Plug
RUN curl -fLo ~/.vim/autoload/plug.vim --create-dirs \
    https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim

# Add file
RUN touch /home/ubuntu/.gitconfig

# Copy .gitconfig
COPY --chown=ubuntu:ubuntu scripts/data/.gitconfig /home/ubuntu/.gitconfig

CMD ["/usr/bin/bash"]
