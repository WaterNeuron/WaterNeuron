# WaterNeuron
Liquid staking protocol on the Internet Computer Protocol.

RUSTFLAGS="--remap-path-prefix=/home/enzo/code/WaterNeuron=/waterNeuron" cargo canister -p water_neuron --bin water_neuron --release --locked

RUSTFLAGS="--remap-path-prefix=/home/enzo/code/WaterNeuron= --remap-path-prefix=/home/enzo/.cargo=" cargo canister -p water_neuron --bin water_neuron --release --locked

nix develop -i -k $HOME

nix develop -i -k HOME -c bash -c "cargo canisters"

## Building
You need to run a Unix based system with x86_64 architecture to compile the canisters.

```bash
# packages to install
sudo apt-get install podman git
# build canisters
./build.sh
# build canisters along with the container image locally
./build.sh --local
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
