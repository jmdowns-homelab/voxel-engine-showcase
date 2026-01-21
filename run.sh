#!/bin/bash
target=$1
if [ "$target" = "dev" ] || [ -z "$target" ]; then
    RUST_LOG=voxel_engine=info cargo run-dev
elif [ "$target" = "staging" ]; then
    RUST_LOG=voxel_engine=info cargo run-staging
elif [ "$target" = "release" ]; then
    cargo run --release
else
    echo "Unknown target, use dev, staging or release!"
fi

