#!/usr/bin/env bash

set -e -o pipefail

if [[ "$1" == "--release" ]]; then
    echo "Building in --release mode"
    TRUNK_ARGS="--release"
    WORKER_BUILD_ARGS="--release"
elif [[ "$1" == "--dev" ]]; then
    echo "Building in --dev mode"
    TRUNK_ARGS=""
    if [[ ! -z "$B2" ]]; then
        echo "Using b2 storage"
        WORKER_BUILD_ARGS="--dev"
    else
        WORKER_BUILD_ARGS="--dev -- --features use-kv-storage"
    fi
else
    echo "expected --release or --dev"
    exit 1
fi

trunk build $TRUNK_ARGS
cd worker
worker-build $WORKER_BUILD_ARGS

cd ..
