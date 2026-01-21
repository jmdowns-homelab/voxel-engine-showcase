RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' \
  cargo +nightly build --target wasm32-unknown-unknown --release -Z build-std=std,panic_abort

wasm-bindgen \
  target/wasm32-unknown-unknown/release/voxel_engine.wasm \
  --out-dir pkg \
  --target web \
  --no-typescript

docker --context beelink cp pkg/voxel_engine_bg.wasm assets-api:assets/voxel_engine_bg.wasm
docker --context beelink cp pkg/voxel_engine.js assets-api:assets/voxel_engine.js
docker --context beelink cp assets/shaders assets-api:assets

cd ../jmdowns.net || exit
google-chrome http://localhost:5173 &
bun run dev