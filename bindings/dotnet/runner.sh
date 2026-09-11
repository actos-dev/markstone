#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MARKSTONE_ROOT="$(cd "$DIR/../.." && pwd)"

export MARKSTONE_NATIVE_DIR="${MARKSTONE_NATIVE_DIR:-$MARKSTONE_ROOT/target/release}"

DLL="$DIR/tests/Markstone.ConformanceRunner/bin/Release/net8.0/Markstone.ConformanceRunner.dll"
if [ ! -f "$DLL" ]; then
    DLL="$DIR/tests/Markstone.ConformanceRunner/bin/Debug/net8.0/Markstone.ConformanceRunner.dll"
fi

if [ ! -f "$DLL" ]; then
    dotnet build "$DIR/tests/Markstone.ConformanceRunner/Markstone.ConformanceRunner.csproj" -c Release > /dev/null 2>&1
    DLL="$DIR/tests/Markstone.ConformanceRunner/bin/Release/net8.0/Markstone.ConformanceRunner.dll"
fi

exec dotnet "$DLL" "$@"
