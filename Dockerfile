FROM --platform=linux/arm64 rust:1.80.1-slim-bullseye AS builder

RUN rustup target install wasm32-unknown-unknown
RUN cargo install wasm-bindgen-cli

WORKDIR /srv/

COPY Cargo.toml /srv/Cargo.toml
COPY Cargo.lock /srv/Cargo.lock
COPY src /srv/src

RUN cargo build --release --target wasm32-unknown-unknown
RUN wasm-bindgen --no-typescript --target web \
    --out-dir ./out/ \
    --out-name "game-of-life" \
    ./target/wasm32-unknown-unknown/release/game-of-life.wasm

FROM --platform=linux/amd64 nginx:1.25.4-bookworm

COPY index.html /usr/share/nginx/html/index.html

COPY --from=builder /srv/out /usr/share/nginx/html/out
