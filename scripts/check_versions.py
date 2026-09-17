#!/usr/bin/env python3
"""Assert that every packaging manifest carries the exact same markstone version.

The version is duplicated by necessity across ecosystem manifests:
Cargo workspace, Rust crates, C ABI header, Python pyproject & Cargo,
Node package.json, JVM pom.xml, .NET csproj, and Go PackageVersion constant.

This script mechanically verifies that all manifests are in exact agreement.

Usage:
    python3 scripts/check_versions.py
"""

import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def read_file(rel_path):
    full = os.path.join(ROOT, rel_path)
    if not os.path.exists(full):
        return None
    with open(full, "r", encoding="utf-8") as f:
        return f.read()


def extract_root_cargo_version():
    content = read_file("Cargo.toml")
    if content is None:
        raise SystemExit("Cargo.toml not found")
    # Look for version under [workspace.package]
    in_workspace_pkg = False
    for line in content.splitlines():
        stripped = line.strip()
        if stripped == "[workspace.package]":
            in_workspace_pkg = True
            continue
        elif stripped.startswith("[") and in_workspace_pkg:
            in_workspace_pkg = False
        if in_workspace_pkg:
            m = re.match(r'^version\s*=\s*"([^"]+)"', stripped)
            if m:
                return m.group(1)
    raise SystemExit("no version found in [workspace.package] in Cargo.toml")


def extract_cargo_member_version(rel_path, root_version):
    content = read_file(rel_path)
    if content is None:
        raise SystemExit(f"{rel_path} not found")

    pkg_version = None
    in_package = False
    for line in content.splitlines():
        stripped = line.strip()
        if stripped == "[package]":
            in_package = True
            continue
        elif stripped.startswith("[") and in_package:
            in_package = False

        if in_package:
            if re.match(r'^version\.workspace\s*=\s*true', stripped):
                pkg_version = root_version
            else:
                m = re.match(r'^version\s*=\s*"([^"]+)"', stripped)
                if m:
                    pkg_version = m.group(1)

    if pkg_version is None:
        raise SystemExit(f"no package version found in {rel_path}")

    # Also verify any sibling markstone dependency versions specified in this Cargo.toml
    dep_version_mismatches = []
    for line in content.splitlines():
        stripped = line.strip()
        m_dep = re.search(r'markstone-(?:core|actos)\s*=\s*\{[^}]*version\s*=\s*"([^"]+)"', stripped)
        if m_dep:
            dep_v = m_dep.group(1)
            if dep_v != root_version:
                dep_version_mismatches.append(f"dependency version {dep_v} != {root_version}")

    if dep_version_mismatches:
        return f"{pkg_version} (dependency mismatch: {', '.join(dep_version_mismatches)})"

    return pkg_version


def extract_regex_version(rel_path, pattern):
    content = read_file(rel_path)
    if content is None:
        raise SystemExit(f"{rel_path} not found")
    for line in content.splitlines():
        m = re.search(pattern, line.strip())
        if m:
            return m.group(1)
    raise SystemExit(f"no version found matching pattern in {rel_path}")


def main():
    root_version = extract_root_cargo_version()

    # Define all required manifest checks
    manifest_checkers = {
        "Cargo.toml": lambda: root_version,
        "core/Cargo.toml": lambda: extract_cargo_member_version("core/Cargo.toml", root_version),
        "actos/Cargo.toml": lambda: extract_cargo_member_version("actos/Cargo.toml", root_version),
        "abi/Cargo.toml": lambda: extract_cargo_member_version("abi/Cargo.toml", root_version),
        "abi/include/markstone.h": lambda: extract_regex_version(
            "abi/include/markstone.h", r'^#define\s+MARKSTONE_VERSION\s+"([^"]+)"'
        ),
        "bindings/rust/Cargo.toml": lambda: extract_cargo_member_version("bindings/rust/Cargo.toml", root_version),
        "bindings/python/Cargo.toml": lambda: extract_cargo_member_version("bindings/python/Cargo.toml", root_version),
        "bindings/python/pyproject.toml": lambda: extract_regex_version(
            "bindings/python/pyproject.toml", r'^version\s*=\s*"([^"]+)"'
        ),
        "bindings/node/package.json": lambda: extract_regex_version(
            "bindings/node/package.json", r'^"version"\s*:\s*"([^"]+)"'
        ),
        "bindings/node/browser.js": lambda: extract_regex_version(
            "bindings/node/browser.js", r"^export const version = '([^']+)';"
        ),
        "bindings/jvm/pom.xml": lambda: extract_regex_version(
            "bindings/jvm/pom.xml", r'^<version>([^<]+)</version>'
        ),
        "bindings/dotnet/src/Markstone/Markstone.csproj": lambda: extract_regex_version(
            "bindings/dotnet/src/Markstone/Markstone.csproj", r'^<Version>([^<]+)</Version>'
        ),
        "bindings/go/markstone.go": lambda: extract_regex_version(
            "bindings/go/markstone.go", r'^const\s+PackageVersion\s*=\s*"([^"]+)"'
        ),
    }

    # Optional check: bindings/go/internal/loader/loader.go if it exists
    loader_go = os.path.join(ROOT, "bindings/go/internal/loader/loader.go")
    if os.path.exists(loader_go):
        manifest_checkers["bindings/go/internal/loader/loader.go"] = lambda: extract_regex_version(
            "bindings/go/internal/loader/loader.go", r'^const\s+PackageVersion\s*=\s*"([^"]+)"'
        )

    versions = {}
    for path, checker in manifest_checkers.items():
        versions[path] = checker()

    expected = root_version
    mismatched = {p: v for p, v in versions.items() if v != expected}

    if mismatched:
        print(f"version mismatch (expected {expected} from Cargo.toml):", file=sys.stderr)
        for path, found in sorted(mismatched.items()):
            print(f"  {path}: {found}", file=sys.stderr)
        return 1

    print(f"version {expected} consistent across {len(versions)} manifest(s):")
    for path, v in sorted(versions.items()):
        print(f"  {path}: {v}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
