#!/usr/bin/env bash

cargo build --bin fxc-server --release --target x86_64-pc-windows-msvc

if [[ ! -f "target/x86_64-pc-windows-msvc/release/d3dcompiler_47.dll" ]]; then
    cp "tools/d3dcompiler_47.dll" "target/x86_64-pc-windows-msvc/release"
fi
