# WaterNeuron
Liquid staking protocol on the Internet Computer Protocol.


## Reproducible Builds
You need to run a Unix based system with x86_64 architecture with Nix installed to compile the canisters.
We recommend the Determinate Systems nix install tool which you can find [here](https://determinate.systems/posts/determinate-nix-installer/).

```bash
# build canisters
./build.sh
```

```bash
# run tests
nix develop
cargo t
```
