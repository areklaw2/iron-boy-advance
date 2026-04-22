desktop rom *flags:
  cargo run --release --bin desktop -- --rom {{rom}} {{flags}}

desktop-bios bios rom *flags:
  cargo run --bin desktop -- --bios {{bios}} --rom {{rom}} {{flags}}

run rom *flags:
  cargo run --bin desktop -- --rom {{rom}} {{flags}}

run-bios bios rom *flags:
  cargo run --bin desktop -- --bios {{bios}} --rom {{rom}} {{flags}}

profile rom *flags:
  cargo build --release --bin desktop
  samply record ./target/release/desktop --rom {{rom}} {{flags}}
