#!/bin/bash
cd $(dirname $0)
cargo test && \
wasm-pack test --node && \
wasm-pack build --target nodejs --scope dr666m1 && \
wasm-pack publish --access public
