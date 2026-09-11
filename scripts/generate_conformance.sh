#!/usr/bin/env bash
set -euo pipefail

# Deterministic conformance golden file generation helper
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cargo run --manifest-path "${ROOT_DIR}/conformance/runner/Cargo.toml" --bin generate-conformance -- "$@"
