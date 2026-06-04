run rom *flags:
  cargo run --bin IronBoyAdvance -- --rom "{{rom}}" {{flags}}

run-bios bios rom *flags:
  cargo run --bin IronBoyAdvance -- --bios {{bios}} --rom "{{rom}}" {{flags}}

run-release bios rom *flags:
  cargo run --release --bin IronBoyAdvance -- --bios {{bios}} --rom "{{rom}}" {{flags}}

profile rom *flags:
  cargo build --release --bin IronBoyAdvance
  samply record ./target/release/IronBoyAdvance --rom {{rom}} {{flags}}
