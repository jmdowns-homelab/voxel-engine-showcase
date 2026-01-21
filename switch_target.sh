#!/bin/bash
target=$1
if [ "$target" = "web"  ]; then
    json -I -f ./.vscode/settings.json -e 'this["rust-analyzer.cargo.target"]="wasm32-unknown-unknown"'
elif [ "$target" = "native" ]; then
    json -I -f ./.vscode/settings.json -e 'this["rust-analyzer.cargo.target"]="x86_64-unknown-linux-gnu"'
else
    echo "Unknown target, use web or native!"
fi