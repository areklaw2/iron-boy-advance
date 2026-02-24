dev rom *flags:
  cargo run --release --bin dev -- --rom {{rom}} {{flags}}

dev-bios bios rom *flags:
  cargo run --bin dev -- --bios {{bios}} --rom {{rom}} {{flags}}

run rom *flags:
  cargo run --bin dev -- --rom {{rom}} {{flags}}

run-bios bios rom *flags:
  cargo run --bin dev -- --bios {{bios}} --rom {{rom}} {{flags}}

profile rom *flags:
  cargo build --release --bin dev
  samply record ./target/release/dev --rom {{rom}} {{flags}}
