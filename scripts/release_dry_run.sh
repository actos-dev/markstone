#!/usr/bin/env bash
# ==============================================================================
# markstone — Release Dry Run & Pre-Flight Verification
# ==============================================================================
# Automates a complete pre-flight release test locally:
# 1. Version Consistency Check across all 13 manifests
# 2. Native packaging helper verification
# 3. Conformance verification for all 7 bindings (Rust, Python, Node, WASM, JVM, .NET, Go)
# 4. Packaging validation for Cargo crates (core, actos, markstone)
# 5. Packaging validation for Python wheel & sdist
# 6. Packaging validation for npm package
# 7. Packaging validation for JVM JAR with bundled natives
# 8. Packaging validation for NuGet package with runtimes/
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

BOLD='\033[1m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PORT_FILE="${ROOT_DIR}/target/rehearsal_port.txt"
SERVER_PID=""

cleanup() {
    if [ -n "${SERVER_PID}" ]; then
        kill -9 "${SERVER_PID}" 2>/dev/null || true
    fi
    rm -f "${ROOT_DIR}/.cargo/config.toml" "${PORT_FILE}"
}
trap cleanup EXIT

echo -e "${BOLD}${BLUE}======================================================================${NC}"
echo -e "${BOLD}${BLUE}             MARKSTONE RELEASE PRE-FLIGHT DRY RUN                     ${NC}"
echo -e "${BOLD}${BLUE}======================================================================${NC}"

PASSED_STEPS=()
FAILED_STEPS=()

record_success() {
    echo -e "${GREEN}✓ [PASS] $1${NC}"
    PASSED_STEPS+=("$1")
}

record_failure() {
    echo -e "${RED}✗ [FAIL] $1${NC}"
    FAILED_STEPS+=("$1")
}

# ------------------------------------------------------------------------------
# 1. Version Consistency Check
# ------------------------------------------------------------------------------
echo -e "\n${BOLD}[1/8] Verifying version consistency across all manifests...${NC}"
if python3 "${ROOT_DIR}/scripts/check_versions.py"; then
    record_success "Version Consistency Check"
else
    record_failure "Version Consistency Check"
    exit 1
fi

# ------------------------------------------------------------------------------
# 2. Arrange Native Libraries
# ------------------------------------------------------------------------------
echo -e "\n${BOLD}[2/8] Arranging and verifying native libraries...${NC}"
cargo build --release -p markstone-abi
cargo build --release -p markstone-node
python3 "${ROOT_DIR}/scripts/pack_natives.py" --artifacts "${ROOT_DIR}/target/release"
if python3 "${ROOT_DIR}/scripts/pack_natives.py" --artifacts "${ROOT_DIR}/target/release" --check; then
    record_success "Native Distribution Layout Check"
else
    record_failure "Native Distribution Layout Check"
fi

# ------------------------------------------------------------------------------
# 3. Conformance Suite Across All 7 Bindings
# ------------------------------------------------------------------------------
echo -e "\n${BOLD}[3/8] Running conformance runner across all 7 bindings...${NC}"
cargo build --release -p markstone-conformance --bin markstone-conformance-runner
RUNNER="${ROOT_DIR}/target/release/markstone-conformance-runner"

# 3.1 Internal Rust crates
echo -e "  -> Checking 1/7: Internal Rust (markstone-core & markstone-actos)..."
if "${RUNNER}" > /dev/null 2>&1; then
    record_success "Conformance: Internal Rust"
else
    record_failure "Conformance: Internal Rust"
fi

# 3.2 Python binding
echo -e "  -> Checking 2/7: Python binding..."
if "${RUNNER}" --exec "python3 ${ROOT_DIR}/bindings/python/runner.py" > /dev/null 2>&1; then
    record_success "Conformance: Python"
else
    record_failure "Conformance: Python"
fi

# 3.3 Node native addon
echo -e "  -> Checking 3/7: Node native addon..."
if "${RUNNER}" --exec "node ${ROOT_DIR}/bindings/node/runner-node.js" > /dev/null 2>&1; then
    record_success "Conformance: Node.js (native)"
else
    record_failure "Conformance: Node.js (native)"
fi

# 3.4 Browser WASM
echo -e "  -> Checking 4/7: Browser WASM..."
if "${RUNNER}" --exec "node ${ROOT_DIR}/bindings/node/runner-wasm.js" > /dev/null 2>&1; then
    record_success "Conformance: Browser WASM"
else
    record_failure "Conformance: Browser WASM"
fi

# 3.5 JVM binding
echo -e "  -> Checking 5/7: JVM binding..."
if "${RUNNER}" --exec "bash ${ROOT_DIR}/bindings/jvm/runner.sh" > /dev/null 2>&1; then
    record_success "Conformance: JVM (Java & Kotlin)"
else
    record_failure "Conformance: JVM (Java & Kotlin)"
fi

# 3.6 .NET binding
echo -e "  -> Checking 6/7: .NET binding..."
if "${RUNNER}" --exec "bash ${ROOT_DIR}/bindings/dotnet/runner.sh" > /dev/null 2>&1; then
    record_success "Conformance: .NET"
else
    record_failure "Conformance: .NET"
fi

# 3.7 Go binding
echo -e "  -> Checking 7/7: Go binding..."
if "${RUNNER}" --exec "bash ${ROOT_DIR}/bindings/go/runner.sh" > /dev/null 2>&1; then
    record_success "Conformance: Go"
else
    record_failure "Conformance: Go"
fi

# ------------------------------------------------------------------------------
# 4. Crates Packaging Validation (core, actos, markstone)
# ------------------------------------------------------------------------------
echo -e "\n${BOLD}[4/8] Validating crates.io packaging (markstone-core, markstone-actos, markstone)...${NC}"
mkdir -p "${ROOT_DIR}/target"
rm -f "${PORT_FILE}"

python3 "${SCRIPT_DIR}/rehearsal_server.py" "${PORT_FILE}" &
SERVER_PID=$!

for i in {1..30}; do
    if [ -f "${PORT_FILE}" ]; then
        PORT=$(cat "${PORT_FILE}")
        if curl -s "http://127.0.0.1:${PORT}/config.json" > /dev/null 2>&1; then
            break
        fi
    fi
    sleep 0.1
done

PORT=$(cat "${PORT_FILE}")
mkdir -p "${ROOT_DIR}/.cargo"
cat << CONFIG_EOF > "${ROOT_DIR}/.cargo/config.toml"
[source.crates-io]
replace-with = "rehearsal"

[source.rehearsal]
registry = "sparse+http://127.0.0.1:${PORT}/"
CONFIG_EOF

cargo package -p markstone-core --allow-dirty --no-verify
cargo package -p markstone-actos --allow-dirty --no-verify
(cd "${ROOT_DIR}/bindings/rust" && cargo package --allow-dirty --no-verify)

if [ -f "${ROOT_DIR}/target/package/markstone-core-0.1.0.crate" ] && \
   [ -f "${ROOT_DIR}/target/package/markstone-actos-0.1.0.crate" ] && \
   [ -f "${ROOT_DIR}/target/package/markstone-0.1.0.crate" ]; then
    record_success "Crates Packaging (core, actos, markstone)"
else
    record_failure "Crates Packaging (core, actos, markstone)"
fi

kill -9 "${SERVER_PID}" 2>/dev/null || true
SERVER_PID=""
rm -f "${ROOT_DIR}/.cargo/config.toml" "${PORT_FILE}"

# ------------------------------------------------------------------------------
# 5. Python Wheel & Sdist Packaging
# ------------------------------------------------------------------------------
echo -e "\n${BOLD}[5/8] Validating Python wheel and sdist packaging...${NC}"
(cd "${ROOT_DIR}/bindings/python" && maturin build --release > /dev/null 2>&1 && maturin sdist > /dev/null 2>&1)
if ls "${ROOT_DIR}/target/wheels"/markstone-*.whl > /dev/null 2>&1 && \
   ls "${ROOT_DIR}/target/wheels"/markstone-*.tar.gz > /dev/null 2>&1; then
    record_success "Python Packaging (wheel + sdist)"
else
    record_failure "Python Packaging (wheel + sdist)"
fi

# ------------------------------------------------------------------------------
# 6. npm Packaging Validation
# ------------------------------------------------------------------------------
echo -e "\n${BOLD}[6/8] Validating npm packaging (dry-run)...${NC}"
if (cd "${ROOT_DIR}/bindings/node" && npm pack --dry-run > /dev/null 2>&1); then
    record_success "npm Packaging (native addon + browser wasm)"
else
    record_failure "npm Packaging (native addon + browser wasm)"
fi

# ------------------------------------------------------------------------------
# 7. JVM JAR Packaging Validation
# ------------------------------------------------------------------------------
echo -e "\n${BOLD}[7/8] Validating JVM jar packaging with bundled native libraries...${NC}"
if [ -d "/usr/lib/jvm/java-25-openjdk" ]; then
    export JAVA_HOME="/usr/lib/jvm/java-25-openjdk"
elif [ -d "/usr/lib/jvm/java-22-openjdk" ]; then
    export JAVA_HOME="/usr/lib/jvm/java-22-openjdk"
fi

(cd "${ROOT_DIR}/bindings/jvm" && mvn -B -DskipTests package > /dev/null 2>&1)
JAR_PATH="${ROOT_DIR}/bindings/jvm/target/markstone-0.1.0.jar"
if [ -f "${JAR_PATH}" ] && jar tf "${JAR_PATH}" | grep -q "native/"; then
    record_success "JVM Packaging (JAR with native libraries)"
else
    record_failure "JVM Packaging (JAR with native libraries)"
fi

# ------------------------------------------------------------------------------
# 8. .NET NuGet Packaging Validation
# ------------------------------------------------------------------------------
echo -e "\n${BOLD}[8/8] Validating .NET NuGet packaging with runtimes/ native libraries...${NC}"
DOTNET_PKG_DIR="${ROOT_DIR}/bindings/dotnet/artifacts"
mkdir -p "${DOTNET_PKG_DIR}"
dotnet pack "${ROOT_DIR}/bindings/dotnet/src/Markstone/Markstone.csproj" -c Release -o "${DOTNET_PKG_DIR}" > /dev/null 2>&1

NUPKG_PATH="${DOTNET_PKG_DIR}/Markstone.0.1.0.nupkg"
if [ -f "${NUPKG_PATH}" ] && unzip -l "${NUPKG_PATH}" | grep -q "runtimes/"; then
    record_success "NuGet Packaging (nupkg with runtimes/)"
else
    record_failure "NuGet Packaging (nupkg with runtimes/)"
fi

# ------------------------------------------------------------------------------
# Summary Report
# ------------------------------------------------------------------------------
echo -e "\n${BOLD}${BLUE}======================================================================${NC}"
echo -e "${BOLD}${BLUE}                      RELEASE DRY RUN SUMMARY                         ${NC}"
echo -e "${BOLD}${BLUE}======================================================================${NC}"
echo -e "Passed Steps: ${#PASSED_STEPS[@]}"
echo -e "Failed Steps: ${#FAILED_STEPS[@]}"

if [ ${#FAILED_STEPS[@]} -eq 0 ]; then
    echo -e "\n${GREEN}${BOLD}✓ ALL PRE-FLIGHT RELEASE CHECKS PASSED SUCCESSFULLY!${NC}\n"
    exit 0
else
    echo -e "\n${RED}${BOLD}✗ SOME CHECKS FAILED:${NC}"
    for step in "${FAILED_STEPS[@]}"; do
        echo -e "  - ${RED}${step}${NC}"
    done
    echo ""
    exit 1
fi
