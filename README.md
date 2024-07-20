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

### Verify

[Proposal#7]()
```bash
bazel run //:verify -- -p 7 -w cac207cf438df8c9fba46d4445c097f05fd8228a1eeacfe0536b7e9ddefc5f1c -c index-canister -g a3831c87440df4821b435050c8a8fcb3745d86f6 -t btxkl-saaaa-aaaar-qagvq-cai -u '(opt IndexArg)' '(opt variant{Init=record{ledger_id=principal "buwm7-7yaaa-aaaar-qagva-cai"}})'
```

### Find file location

```bash
ubuntu@arrakis:~/waterneuron$ bazel query --output=location @ledger-canister//:ledger.did
/home/ubuntu/.cache/bazel/_bazel_ubuntu/d47e8bc570bf9aeab12a6465bf3a11fb/external/ledger-canister/ledger.did:1:1: source file @ledger-canister//:ledger.did
```


