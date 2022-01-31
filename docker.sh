#!/usr/bin/env bash

set -euo pipefail

docker build -t pasteofexile .
exec docker run --rm -it -v $PWD:/pasteofexile -p 8787:8787 -u "$(id -u):$(id -g)" pasteofexile yarn start
