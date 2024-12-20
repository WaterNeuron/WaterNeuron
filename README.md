# WaterNeuron
Liquid staking protocol on the Internet Computer Protocol.

## Building
You need to run a Unix based system with x86_64 architecture to compile the canisters.

```bash
# packages to install
sudo apt-get install podman git
# build canisters
./run.sh
# build canisters along with the container image locally
./run.sh --local
```

## Dev
You need the following packages to run system tests.

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
