#!/usr/bin/env bash

# Find the host target triple by parsing rustc version output.
host_triple=`rustc --version -v | grep host: | sed 's/host: //'`

# Build and run xtask binary, explicitly passing the target triple.
# This is necessary to avoid build cache invalidations when cross compiling,
# due to issues with how cargo's cache tagging interacts with RUSTFLAGS changes.
# https://github.com/rust-lang/cargo/issues/9239
cargo run -p xtask --target $host_triple -- $@
