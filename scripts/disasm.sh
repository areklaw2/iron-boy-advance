#!/usr/bin/env bash
set -euo pipefail

show_raw=--no-show-raw-insn
bin=/tmp/block.bin

for arg in "$@"; do
  case "$arg" in
    --raw) show_raw= ;;
    *) bin="$arg" ;;
  esac
done

obj="$(mktemp -d)/block.o"
tools="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/bin"

"$tools/llvm-objcopy" -I binary -O elf64-littleaarch64 --rename-section=.data=.text,code "$bin" "$obj"
"$tools/llvm-objdump" -d $show_raw "$obj"
