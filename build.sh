#!/bin/bash
mkdir -p www
cargo build --target wasm32-unknown-unknown
cp index.html www/
cp target/wasm32-unknown-unknown/debug/blackjack.wasm www/
cp -R assets/ www/assets/
