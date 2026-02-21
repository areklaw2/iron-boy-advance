dev rom:
  cargo run --bin dev -- --rom {{rom}} --logs

dev-bios bios rom:
  cargo run --bin dev -- --bios {{bios}} --rom {{rom}} --logs
