#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN="$DIR/bin/runner"

if [ ! -f "$BIN" ] || [ "$DIR/cmd/runner/main.go" -nt "$BIN" ]; then
    mkdir -p "$DIR/bin"
    (cd "$DIR" && CGO_ENABLED=0 go build -o "$BIN" ./cmd/runner)
fi

exec "$BIN" "$@"
