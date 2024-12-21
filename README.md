# WaterNeuron
Liquid staking protocol on the Internet Computer Protocol.


## Reproducible Builds
You need to run a Unix based system with x86_64 architecture with Nix installed to compile the canisters.
We recommend the Determinate Systems nix install tool which you can find [here](https://determinate.systems/posts/determinate-nix-installer/).

```bash
# build canisters
nix develop -i -k HOME -c bash -c "cargo canisters"
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
