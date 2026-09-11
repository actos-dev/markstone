#!/usr/bin/env python3
"""Arrange built native libraries into ecosystem-specific package layouts.

The JVM, .NET, Go, and Node packages distribute prebuilt binaries:
- JVM:   bindings/jvm/native/{os}-{arch}/<lib> (and src/main/resources/native/{os}-{arch}/<lib>)
- .NET:  bindings/dotnet/src/Markstone/runtimes/{rid}/native/<lib>
- Go:    bindings/go/internal/embeds/<lib>
- Node:  bindings/node/markstone.node and bindings/node/wasm/

Usage:
    # Arrange from release directory or downloaded artifacts:
    python3 scripts/pack_natives.py --artifacts target/release
    python3 scripts/pack_natives.py --artifacts dist/natives --check

Options:
    --artifacts PATH   Directory where native artifacts or builds are located (required)
    --check            Verify that target layouts exist and match without writing
    --strict           Require all supported platforms to be present
"""

import argparse
import os
import platform
import shutil
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

JVM_NATIVE_ROOT = os.path.join(ROOT, "bindings", "jvm", "native")
JVM_RESOURCES_NATIVE_ROOT = os.path.join(ROOT, "bindings", "jvm", "src", "main", "resources", "native")
DOTNET_RUNTIMES_ROOT = os.path.join(ROOT, "bindings", "dotnet", "src", "Markstone", "runtimes")
GO_EMBEDS_ROOT = os.path.join(ROOT, "bindings", "go", "internal", "embeds")
NODE_ROOT = os.path.join(ROOT, "bindings", "node")

# Target definitions
# (rid, target_triple, lib_candidates, jvm_dir, dotnet_rid, go_embed_name)
TARGETS = [
    {
        "rid": "linux-x64",
        "triple": "x86_64-unknown-linux-gnu",
        "libs": ["libmarkstone_abi.so", "libmarkstone.so"],
        "jvm_dir": "linux-x86_64",
        "dotnet_rid": "linux-x64",
        "go_embed": "libmarkstone_abi_linux_amd64.so",
    },
    {
        "rid": "linux-musl-x64",
        "triple": "x86_64-unknown-linux-musl",
        "libs": ["libmarkstone_abi.so", "libmarkstone.so"],
        "jvm_dir": None,
        "dotnet_rid": "linux-musl-x64",
        "go_embed": None,
    },
    {
        "rid": "linux-arm64",
        "triple": "aarch64-unknown-linux-gnu",
        "libs": ["libmarkstone_abi.so", "libmarkstone.so"],
        "jvm_dir": "linux-aarch64",
        "dotnet_rid": "linux-arm64",
        "go_embed": None,
    },
    {
        "rid": "linux-musl-arm64",
        "triple": "aarch64-unknown-linux-musl",
        "libs": ["libmarkstone_abi.so", "libmarkstone.so"],
        "jvm_dir": None,
        "dotnet_rid": "linux-musl-arm64",
        "go_embed": None,
    },
    {
        "rid": "osx-x64",
        "triple": "x86_64-apple-darwin",
        "libs": ["libmarkstone_abi.dylib", "libmarkstone.dylib"],
        "jvm_dir": "macos-x86_64",
        "dotnet_rid": "osx-x64",
        "go_embed": None,
    },
    {
        "rid": "osx-arm64",
        "triple": "aarch64-apple-darwin",
        "libs": ["libmarkstone_abi.dylib", "libmarkstone.dylib"],
        "jvm_dir": "macos-aarch64",
        "dotnet_rid": "osx-arm64",
        "go_embed": None,
    },
    {
        "rid": "win-x64",
        "triple": "x86_64-pc-windows-msvc",
        "libs": ["markstone_abi.dll", "markstone.dll"],
        "jvm_dir": "windows-x86_64",
        "dotnet_rid": "win-x64",
        "go_embed": None,
    },
    {
        "rid": "win-arm64",
        "triple": "aarch64-pc-windows-msvc",
        "libs": ["markstone_abi.dll", "markstone.dll"],
        "jvm_dir": "windows-aarch64",
        "dotnet_rid": "win-arm64",
        "go_embed": None,
    },
]


def is_host_target(rid):
    current_os = platform.system().lower()
    current_arch = platform.machine().lower()
    if "linux" in current_os and current_arch in ("x86_64", "amd64"):
        return rid == "linux-x64"
    if "linux" in current_os and current_arch in ("aarch64", "arm64"):
        return rid == "linux-arm64"
    if "darwin" in current_os and current_arch in ("aarch64", "arm64"):
        return rid == "osx-arm64"
    if "darwin" in current_os and current_arch in ("x86_64", "amd64"):
        return rid == "osx-x64"
    if "windows" in current_os and current_arch in ("x86_64", "amd64"):
        return rid == "win-x64"
    if "windows" in current_os and current_arch in ("aarch64", "arm64"):
        return rid == "win-arm64"
    return False


def find_library_file(artifacts, target_info):
    """Find the compiled native library in artifacts for the given target."""
    rid = target_info["rid"]
    triple = target_info["triple"]
    libs = target_info["libs"]

    # Explicit subdirectories for this target
    candidate_subdirs = [
        os.path.join(artifacts, f"native-{rid}"),
        os.path.join(artifacts, f"native-{triple}"),
        os.path.join(artifacts, rid),
        os.path.join(artifacts, triple),
    ]
    if target_info.get("jvm_dir"):
        candidate_subdirs.append(os.path.join(artifacts, target_info["jvm_dir"]))

    for subdir in candidate_subdirs:
        if not os.path.isdir(subdir):
            continue
        for lib in libs:
            p = os.path.join(subdir, lib)
            if os.path.isfile(p):
                return p

    # If flat directory, only match if this target matches the host platform
    if is_host_target(rid):
        for lib in libs:
            p = os.path.join(artifacts, lib)
            if os.path.isfile(p):
                return p

    return None


def place_file(src, dst, check_mode):
    """Copies src to dst, or checks size if check_mode is True."""
    if check_mode:
        if not os.path.isfile(dst):
            return f"{os.path.relpath(dst, ROOT)}: missing"
        if os.path.getsize(dst) != os.path.getsize(src):
            return f"{os.path.relpath(dst, ROOT)}: size mismatch with source {os.path.relpath(src, ROOT)}"
        return None

    os.makedirs(os.path.dirname(dst), exist_ok=True)
    shutil.copy2(src, dst)
    return None


def pack_target(target, source_file, check_mode):
    """Places source_file into JVM, .NET, and Go packaging locations."""
    problems = []
    placed_count = 0
    primary_lib_name = target["libs"][0]
    fallback_lib_name = target["libs"][1] if len(target["libs"]) > 1 else primary_lib_name

    # 1. JVM destination
    if target["jvm_dir"]:
        jvm_dirs = [
            os.path.join(JVM_NATIVE_ROOT, target["jvm_dir"]),
            os.path.join(JVM_RESOURCES_NATIVE_ROOT, target["jvm_dir"]),
        ]
        for jd in jvm_dirs:
            for fname in [primary_lib_name, fallback_lib_name]:
                dst = os.path.join(jd, fname)
                prob = place_file(source_file, dst, check_mode)
                if prob:
                    problems.append(prob)
                else:
                    placed_count += 1

    # 2. .NET destination
    if target["dotnet_rid"]:
        dotnet_dir = os.path.join(DOTNET_RUNTIMES_ROOT, target["dotnet_rid"], "native")
        for fname in [primary_lib_name, fallback_lib_name]:
            dst = os.path.join(dotnet_dir, fname)
            prob = place_file(source_file, dst, check_mode)
            if prob:
                problems.append(prob)
            else:
                placed_count += 1

    # 3. Go embed destination (if defined and file is referenced)
    if target["go_embed"]:
        go_dst = os.path.join(GO_EMBEDS_ROOT, target["go_embed"])
        prob = place_file(source_file, go_dst, check_mode)
        if prob:
            problems.append(prob)
        else:
            placed_count += 1

    return placed_count, problems


def pack_node_and_wasm(artifacts, check_mode):
    """Places Node native addon and Browser WASM bundle into bindings/node."""
    problems = []
    placed_count = 0

    # Node addon candidates: markstone.node, libmarkstone_node.so/dylib, markstone_node.dll
    node_addon_candidates = [
        os.path.join(artifacts, "markstone.node"),
        os.path.join(artifacts, "libmarkstone_node.so"),
        os.path.join(artifacts, "libmarkstone_node.dylib"),
        os.path.join(artifacts, "markstone_node.dll"),
        os.path.join(artifacts, "native-node", "markstone.node"),
    ]
    node_src = None
    for cand in node_addon_candidates:
        if os.path.isfile(cand):
            node_src = cand
            break

    if node_src:
        node_dst = os.path.join(NODE_ROOT, "markstone.node")
        prob = place_file(node_src, node_dst, check_mode)
        if prob:
            problems.append(prob)
        else:
            placed_count += 1

    # Browser wasm bundle candidates
    wasm_dir_candidates = [
        os.path.join(artifacts, "wasm"),
        os.path.join(artifacts, "native-wasm"),
        os.path.join(artifacts, "wasm32-unknown-unknown"),
        os.path.join(artifacts, "pkg"),
    ]
    wasm_src_dir = None
    for cand in wasm_dir_candidates:
        if os.path.isdir(cand) and os.path.isfile(os.path.join(cand, "markstone_wasm_bg.wasm")):
            wasm_src_dir = cand
            break

    if wasm_src_dir:
        node_wasm_dst = os.path.join(NODE_ROOT, "wasm")
        os.makedirs(node_wasm_dst, exist_ok=True)
        for item in os.listdir(wasm_src_dir):
            s = os.path.join(wasm_src_dir, item)
            d = os.path.join(node_wasm_dst, item)
            if os.path.isfile(s):
                prob = place_file(s, d, check_mode)
                if prob:
                    problems.append(prob)
                else:
                    placed_count += 1

    return placed_count, problems


def main():
    parser = argparse.ArgumentParser(description="Pack native libraries for markstone bindings")
    parser.add_argument("--artifacts", required=True, help="Directory containing prebuilt artifacts")
    parser.add_argument("--check", action="store_true", help="Verify layouts instead of writing")
    parser.add_argument("--strict", action="store_true", help="Require all target platforms to be found")
    args = parser.parse_args()

    if not os.path.isdir(args.artifacts):
        print(f"Error: artifacts directory '{args.artifacts}' does not exist", file=sys.stderr)
        return 1

    all_problems = []
    total_placed = 0
    found_targets = []
    missing_targets = []

    for target in TARGETS:
        source_lib = find_library_file(args.artifacts, target)
        if source_lib is None:
            missing_targets.append(target["rid"])
            if args.strict:
                all_problems.append(f"Missing library for target {target['rid']} ({target['triple']})")
            continue

        found_targets.append(target["rid"])
        placed, problems = pack_target(target, source_file=source_lib, check_mode=args.check)
        total_placed += placed
        all_problems.extend(problems)

    # Also handle Node addon & WASM bundle if present
    node_placed, node_problems = pack_node_and_wasm(args.artifacts, args.check)
    total_placed += node_placed
    all_problems.extend(node_problems)

    if all_problems:
        print("Encountered errors during native packing / verification:", file=sys.stderr)
        for p in all_problems:
            print(f"  {p}", file=sys.stderr)
        return 1

    action = "verified" if args.check else "placed"
    print(f"Native distribution helper: {action} {total_placed} file(s) across {len(found_targets)} target(s)")
    if missing_targets and not args.strict:
        print(f"  Note: {len(missing_targets)} target(s) not found in artifacts ({', '.join(missing_targets)})")

    return 0


if __name__ == "__main__":
    sys.exit(main())
