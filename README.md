# Iron Boy Advance

Iron Boy Advance a Game Boy Advance emulator, written in rust.

## Features

- [ ] Game Boy Advance Components
  - [x] CPU (ARM7TDMI)
  - [ ] Memory
    - [ ] Open Bus
  - [ ] Hardware
    - [x] LCD
    - [ ] Sound
    - [x] Timers
    - [x] DMA Transfers
    - [ ] Communication Ports
      - [ ] Same computer Link Cable support
    - [x] Keypad
    - [ ] Interrupts
    - [x] System Control
    - [ ] Cartridges
      - [ ] Real-time clock support
    - [x] Bios
      - [x] Ability to load External BIOS
- [ ] Game Boy/ Game Boy Color support
- [ ] Just-in-time (JIT) compilation
- [ ] Scheduler based game Loop
- [ ] UI
  - [ ] Desktop frontend
    - [ ] Graphics Views
      - [ ] Palette Viewer
      - [ ] Sprite Viewer
      - [ ] Tile Viewer
      - [ ] Map Viewer
    - [ ] Backround Only Viewer
    - [ ] Window Only Viewer
    - [ ] Audio Channel Visualizer
    - [ ] Log viewer
      - [ ] Searchable disassembler log window
      - [ ] Executed Instruction Log
      - [ ] Exportable log files
    - [ ] Screenshots
  - [ ] WASM frontend
  - [ ] Drag and drop file loading
  - [ ] Game savestates
  - [ ] Fast Forwarding
  - [ ] Pausing

## Testing

- [ARM7TDMI Single Step Test](https://github.com/SingleStepTests/ARM7TDMI) :white_check_mark:
- [gba-tests](https://github.com/jsmolka/gba-tests)
  - arm :white_check_mark:
  - thumb :white_check_mark:
  - bios :white_check_mark:
  - ppu :white_check_mark:
  - nes :white_check_mark:
- [arm-wrestler](https://github.com/destoer/armwrestler-gba-fixed) :white_check_mark:

## License

Iron Boy Advance is licensed under the terms of the GNU General Public License (GPL) 3.0 or any later version. See [LICENSE](LICENSE) for details.

Copyright (C) 2026 Aza Walker
