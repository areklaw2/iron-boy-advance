dev rom *flags:
  cargo run --bin dev -- --rom {{rom}} {{flags}}

dev-bios bios rom *flags:
  cargo run --bin dev -- --bios {{bios}} --rom {{rom}} {{flags}}
