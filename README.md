# game-of-life

# status: working but it's early days

## Native (development)

Note: Dynamic linking is only relevant while you're developing

```shell
cargo run --features bevy/dynamic_linking
```

## WASM (development)

```shell
rustup target install wasm32-unknown-unknown
cargo install wasm-server-runner
cargo install devserver
CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-server-runner cargo run --target wasm32-unknown-unknown
```

## WASM (production)

```shell
cargo install -f wasm-bindgen-cli
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --target web \
    --out-dir ./out/ \
    --out-name "game-of-life" \
    ./target/wasm32-unknown-unknown/release/game-of-life.wasm
devserver
```
