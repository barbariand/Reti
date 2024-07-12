#!/usr/bin bash
# This is a script for running the building and setup so you can use it from the index.html file exported as the window.test function
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
dir=$(pwd);
cd $SCRIPT_DIR
echo "building"
FLAGS="-C opt-level=z -C lto";

RUSTFLAGS=$FLAGS cargo build --release --target wasm32-unknown-unknown
echo "binding wasm"
wasm-bindgen --web $SCRIPT_DIR/../target/wasm32-unknown-unknown/release/Reti_js.wasm --out-dir $SCRIPT_DIR/wasm
echo "done, returning to dir"
cd $dir