#!/usr/bin/env bash
set -euo pipefail

name="$1"
asm="$(mktemp -d)/probe.s"

rustc --edition 2024 -O --emit asm -o "$asm" \
  "crates/ironboyadvance_arm7tdmi_jit/probes/${name}.rs"

# drop directives (.cfi_, .globl, ...) and blanks; tag each line with its
# block's numeric label (rustc's -O output order isn't source order, so a
# stable numeric sort puts the blocks back in order); then strip the tag
# and number each block's instructions from 1
grep -v '^[[:space:]]*\.' "$asm" \
  | grep -v '^[[:space:]]*$' \
  | sed 's/^[[:space:]]*//; s/\t/ /g' \
  | awk '{ if ($0 ~ /:$/) { match($0, /[0-9]+:$/); key = (RSTART ? substr($0, RSTART, RLENGTH - 1) + 0 : 0) } printf "%d\t%s\n", key, $0 }' \
  | sort -t $'\t' -k1,1n -s \
  | awk -F'\t' '{ if ($2 ~ /:$/) { n = 0; print "     " $2 } else { n++; printf "%4d  %s\n", n, $2 } }'
