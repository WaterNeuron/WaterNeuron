# WaterNeuron
Liquid staking protocol on the Internet Computer

## Building
-   We assume you are building on a Ubuntu +22.04 based machine.
-   Packages pre-requisites: `podman`, `git`
```bash
sudo apt-get install podman git
```

-   To test the canister hash of the current commit
```bash
./run.sh --build
```

-   To dev
```bash
./run.sh
```

## Dev

You need the following packages to run the system tests.

```bash
sudo apt udpate && sudo apt install \
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
        liblzma-dev
``
