#!/usr/bin bash
# This is a script for running the building and setup so you can use it from the index.html file exported as the window.test function
cd "$(dirname "$0")"

export RUSTFLAGS="-C opt-level=z"
echo "running with flags $RUSTFLAGS"
cargo build --release --target wasm32-unknown-unknown

echo "binding wasm"
wasm-bindgen --target web ../target/wasm32-unknown-unknown/release/reti_js.wasm --out-dir wasm

echo "done"