
Let's compile `sns_module` from the command line.

```bash
cargo rustc -p sns_module \
    --target wasm32-unknown-unknown \
    --release \
    --bin sns_module \
    -- \
    -C link-args=-z stack-size=3145728 \
    -C linker-plugin-lto \
    -C opt-level=3 \
    -C debug-assertions=no \
    -C debuginfo=0 \
    -C lto
```

```
ic-wasm \
    ./target/wasm32-unknown-unknown/release/sns_module.wasm \
    -o sns_module_with_candid.wasm \
    metadata candid:service -f sns_module/sns_module.did -v public
```
