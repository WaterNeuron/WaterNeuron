
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

## Self-check

```bash
#[cfg(feature = "self_check")]
#[query]
pub fn self_check() {}
```
This will in turn export the function when compiling the canister. with `--feature self_check`

```bash
$ wasm-objdump water_neuron_self_check.wasm -x | rg "canister_query self_check"
 - func[4096] sig=0 <canister_query self_check>
 - func[4096] <canister_query self_check> -> "canister_query self_check"
 - func[4096] size=46 <canister_query self_check>
 ```

When compiling the canister for mainnet the feature flag should not be present, therefore neither should that function.
