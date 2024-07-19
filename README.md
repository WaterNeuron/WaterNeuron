# WaterNeuron
Liquid staking protocol on the Internet Computer

## Building

-   Pre-requisites: `podman`, `git`

## VSCode

Recommend extensions:
- mkhl.direnv
- philipbe.theme-gray-matter
- ms-vscode-remote.remote-containers

## Bazel

Find file location
```
ubuntu@arrakis:~/waterneuron$ bazel query --output=location @ledger-canister//:ledger.did
/home/ubuntu/.cache/bazel/_bazel_ubuntu/d47e8bc570bf9aeab12a6465bf3a11fb/external/ledger-canister/ledger.did:1:1: source file @ledger-canister//:ledger.did
```

## TODO
-   releases
-   ci
-   verify script
-   dfx release script
-   upload to docker image
-   automatically update docker
-   fetch candid files
