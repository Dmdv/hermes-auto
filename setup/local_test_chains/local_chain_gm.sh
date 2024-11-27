#!/bin/bash
set -e

gm="$HOME"/.gm/bin/gm

if [ -z "$gm" ]; then
  warn "missing mandatory gm, install it from https://hermes.informal.systems/tutorials/pre-requisites/index.html"
else
  echo "gm is installed"
fi

gm() {
    if [ $# -eq 2 ]; then
      $gm "$1" "$2"
    else
      $gm "$1"
    fi
}

is_json_output() {
  test "${OUTPUT:-}" = "json"
}

warn() {
  if ! is_json_output; then
    echo "WARNING: $*" >&2
  fi
}

gm keys
gm start
gm hermes config
gm hermes keys
echo "Hermes is configured"
gm status

echo ""
echo "SUCCESS"