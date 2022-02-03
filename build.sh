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
        WORKER_BUILD_ARGS="--dev -- --features debug"
    else
        WORKER_BUILD_ARGS="--dev -- --features debug,storage-kv"
    fi
else
    echo "expected --release or --dev"
    exit 1
fi

trunk build $TRUNK_ARGS
cd worker
worker-build $WORKER_BUILD_ARGS

cat <<EOF > build/worker/assets.mjs
import manifestJSON from '__STATIC_CONTENT_MANIFEST'
const assetManifest = JSON.parse(manifestJSON)

export function get_asset(name) {
    return assetManifest[name];
}
EOF

cd ..
