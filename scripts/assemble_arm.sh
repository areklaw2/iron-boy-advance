#!/usr/bin/env bash
set -euo pipefail

source="$1"
output="$2"

obj="$(mktemp -d)/program.o"
tools="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/bin"

clang --target=armv4t-none-eabi -c "$source" -o "$obj"
"$tools/llvm-objcopy" -O binary --only-section=.text "$obj" "$output"
