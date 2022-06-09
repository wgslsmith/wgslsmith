#!/usr/bin/env bash

set -euo pipefail

cargo build --bin fxc-server --release --target x86_64-pc-windows-msvc

docker build -f Dockerfile.fxc -t fxc-server .
docker run --name fxc-server -p 9123:9123 -d fxc-server -a 0.0.0.0:9123
