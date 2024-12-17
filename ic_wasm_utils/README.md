
Let's compile `sns_module` from the command line.

!! we do not have the canister profile enabled
```bash
cargo canister -p sns_module --release --bin sns_module
```

```
ic-wasm ./target/wasm32-unknown-unknown/release/sns_module.wasm -o artefacts/sns_module_with_candid.wasm metadata candid:service -f sns_module/sns_module.did -v public
```

```
ic-wasm artefacts/sns_module_with_candid.wasm -o artefacts/sns_module_with_candid_and_git.wasm metadata git_commit_id -d $(git rev-parse HEAD) -v public
```

```
ic-wasm artefacts/sns_module_with_candid_and_git.wasm -o artefacts/sns_module_candid_git_shrink.wasm shrink
```

```
gzip -n --force artefacts/sns_module_candid_git_shrink.wasm
```
