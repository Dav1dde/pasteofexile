#!/usr/bin/env bash

set -e -o pipefail

TRUNK_ARGS=""
WORKER_BUILD_ARGS="--dev -- --features debug,storage-kv"
if [[ "$1" == "--release" ]]; then
    echo "Building in --release mode"
    TRUNK_ARGS="--release"
    WORKER_BUILD_ARGS="--release"
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

cat <<EOF > build/worker/export_wasm.mjs
import * as index_bg from "./index_bg.mjs";
import _wasm from "./index_bg.wasm";
let importsObject = {
    "./index_bg.js": index_bg
};
export default new WebAssembly.Instance(_wasm, importsObject).exports;
EOF

cd ..
