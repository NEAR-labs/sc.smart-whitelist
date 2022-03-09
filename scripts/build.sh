#!/bin/bash
set -e

cargo build --target wasm32-unknown-unknown --release
cp ../target/wasm32-unknown-unknown/release/smart_whitelist.wasm ../wasm
