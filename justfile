run *flags:
  cargo run --bin IronBoyAdvance -- {{flags}}

run-rom rom *flags:
  cargo run --bin IronBoyAdvance -- --rom "{{rom}}" {{flags}}

run-bios bios rom *flags:
  cargo run --bin IronBoyAdvance -- --bios {{bios}} --rom "{{rom}}" {{flags}}

run-release bios rom *flags:
  cargo run --release --bin IronBoyAdvance -- --bios {{bios}} --rom "{{rom}}" {{flags}}

profile bios rom:
  cargo build --release --bin IronBoyAdvance
  samply record ./target/release/IronBoyAdvance --bios {{bios}} --rom "{{rom}}"

profile-dev bios rom:
  cargo build --profile profiling --bin IronBoyAdvance
  samply record ./target/profiling/IronBoyAdvance --bios {{bios}} --rom "{{rom}}"

# Disassemble a raw block dump (default /tmp/block.bin); --raw adds the hex encodings.
disasm *args:
  ./scripts/disasm.sh {{args}}

assemble-self-modifying-test:
  ./scripts/assemble_arm.sh crates/ironboyadvance/tests/self_modifying.s crates/ironboyadvance/tests/self_modifying.bin

# Compile one instruction and disassemble it: just dump arm 0xE0234291 [--raw]
dump mode encoding *flags:
  cargo run -q -p ironboyadvance_arm7tdmi_jit --bin dump -- {{mode}} {{encoding}}
  ./scripts/disasm.sh {{flags}}
