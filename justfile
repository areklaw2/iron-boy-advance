run *flags:
  cargo run --bin IronBoyAdvance -- {{flags}}

run-rom rom *flags:
  cargo run --bin IronBoyAdvance -- --rom "{{rom}}" {{flags}}

run-bios bios rom *flags:
  cargo run --bin IronBoyAdvance -- --bios {{bios}} --rom "{{rom}}" {{flags}}

run-release bios rom *flags:
  cargo run --release --bin IronBoyAdvance -- --bios {{bios}} --rom "{{rom}}" {{flags}}

profile bios rom *flags:
  cargo build --release --bin IronBoyAdvance
  samply record ./target/release/IronBoyAdvance --bios {{bios}} --rom "{{rom}}"

profile-dev bios rom *flags:
  cargo build --profile profiling --bin IronBoyAdvance
  samply record ./target/profiling/IronBoyAdvance --bios {{bios}} --rom "{{rom}}"

# Dump the ARM64 rustc generates for a JIT probe, numbered.
probe name:
  #!/usr/bin/env bash
  set -euo pipefail
  asm="$(mktemp -d)/probe.s"
  rustc --edition 2024 -O --emit asm -o "$asm" \
    "crates/ironboyadvance_arm7tdmi_jit/probes/{{name}}.rs"
  # drop directives (.cfi_, .globl, ...) and blanks; number instructions,
  # print labels unnumbered so branch targets stay visible
  grep -v '^[[:space:]]*\.' "$asm" \
    | grep -v '^[[:space:]]*$' \
    | sed 's/^[[:space:]]*//; s/\t/ /g' \
    | awk '/:$/ { printf "     %s\n", $0; next } { printf "%4d  %s\n", ++n, $0 }'
