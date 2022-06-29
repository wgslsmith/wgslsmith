#!/usr/bin/env bash

set -euo pipefail

cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null

cargo build --bin validation-server --release --target x86_64-pc-windows-msvc

docker build -f Dockerfile.validation-server -t wgslsmith-validation-server .
docker run --name wgslsmith-validation-server -p 9123:9123 -d wgslsmith-validation-server -a 0.0.0.0:9123 -q
