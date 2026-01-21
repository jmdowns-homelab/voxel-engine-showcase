#!/bin/sh -e
target=$1
RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' \
  cargo +nightly build --target wasm32-unknown-unknown --release -Z build-std=std,panic_abort

wasm-bindgen \
  target/wasm32-unknown-unknown/release/voxel_engine.wasm \
  --out-dir pkg \
  --target web \
  --no-typescript

#RUST_LOG=voxel_engine=info wasm-pack build --dev --no-typescript --no-pack --target=web --$1 --features gpu_queries
cp -r pkg/ ../../svelte/jmdowns.net/static/
rm -rf pkg
cd ../jmdowns.net
google-chrome http://localhost:5173 &
bun run dev