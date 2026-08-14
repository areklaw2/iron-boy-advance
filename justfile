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
