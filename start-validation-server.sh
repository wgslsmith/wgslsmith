#!/usr/bin/env bash

set -euo pipefail

cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null

# Set environment variables for Windows cross-compilation
export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS="\
    -C linker=$LLVM_NATIVE_TOOLCHAIN/bin/lld-link \
    -Lnative=$XWIN_CACHE/splat/crt/lib/x86_64 \
    -Lnative=$XWIN_CACHE/splat/sdk/lib/ucrt/x86_64 \
    -Lnative=$XWIN_CACHE/splat/sdk/lib/um/x86_64"
export CXX_x86_64_pc_windows_msvc="$LLVM_NATIVE_TOOLCHAIN/bin/clang-cl"
export CXXFLAGS_x86_64_pc_windows_msvc="\
    /imsvc $XWIN_CACHE/splat/crt/include \
    /imsvc $XWIN_CACHE/splat/sdk/include/ucrt"
export AR_x86_64_pc_windows_msvc="$LLVM_NATIVE_TOOLCHAIN/bin/llvm-lib"

# Compile validation server for Windows
cargo build --bin validation-server --release --target x86_64-pc-windows-msvc --target-dir cross-target

# Build container
docker build -f Dockerfile.validation-server -t wgslsmith-validation-server .

# Remove existing container
docker container ls -q --filter name=wgslsmith-validation-server | grep -q . && docker stop wgslsmith-validation-server
docker container ls -qa --filter name=wgslsmith-validation-server | grep -q . && docker rm wgslsmith-validation-server

# Start container
docker run --name wgslsmith-validation-server -p 9123:9123 -d wgslsmith-validation-server -a 0.0.0.0:9123 -q
