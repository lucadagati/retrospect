#!/bin/bash
# Build go-telemetry with TinyGo 0.34 via Docker.
# Mounts the host Go module cache so dependencies are not re-downloaded.
set -e

GOMODCACHE="${GOPATH:-$HOME/go}/pkg/mod"

docker run --rm \
  -v "$(pwd)":/src \
  -v "${GOMODCACHE}":/root/go/pkg/mod \
  -w /src \
  tinygo/tinygo:0.34.0 \
  tinygo build -target=wasip1 -gc=leaking -buildmode=c-shared -no-debug -o main.wasm .
