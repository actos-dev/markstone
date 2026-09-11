#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
PORT_FILE="${ROOT_DIR}/target/rehearsal_port.txt"
mkdir -p "${ROOT_DIR}/target"
rm -f "${PORT_FILE}"

echo "=== Starting Publish Rehearsal ==="

# 1. Start local rehearsal sparse index server on ephemeral port
echo "[1/4] Starting local sparse registry server..."
python3 "${SCRIPT_DIR}/rehearsal_server.py" "${PORT_FILE}" &
SERVER_PID=$!
trap "kill -9 ${SERVER_PID} 2>/dev/null || true; rm -f '${ROOT_DIR}/.cargo/config.toml' '${PORT_FILE}'" EXIT

# Wait for server to write port and be ready
for i in {1..30}; do
    if [ -f "${PORT_FILE}" ]; then
        PORT=$(cat "${PORT_FILE}")
        if curl -s "http://127.0.0.1:${PORT}/config.json" > /dev/null 2>&1; then
            echo "Server ready on port ${PORT}"
            break
        fi
    fi
    sleep 0.1
done

PORT=$(cat "${PORT_FILE}")

# Configure Cargo to use rehearsal registry for crates.io replacement
mkdir -p "${ROOT_DIR}/.cargo"
cat << CONFIG_EOF > "${ROOT_DIR}/.cargo/config.toml"
[source.crates-io]
replace-with = "rehearsal"

[source.rehearsal]
registry = "sparse+http://127.0.0.1:${PORT}/"
CONFIG_EOF

# 2. Package core and actos dependencies
echo "[2/4] Packaging markstone-core..."
cargo package -p markstone-core --allow-dirty --no-verify

echo "[3/4] Packaging markstone-actos..."
cargo package -p markstone-actos --allow-dirty --no-verify

# 3. Package markstone Rust binding
echo "[4/4] Packaging markstone (Rust binding in bindings/rust)..."
(cd "${ROOT_DIR}/bindings/rust" && cargo package --allow-dirty --no-verify)

echo "=== crates.io Publish Rehearsal Succeeded ==="
ls -lh "${ROOT_DIR}/target/package"/*.crate

echo "=== PyPI Publish Rehearsal ==="
(cd "${ROOT_DIR}/bindings/python" && maturin build && maturin sdist)
echo "=== PyPI Publish Rehearsal Succeeded ==="
ls -lh "${ROOT_DIR}/target/wheels"/*
