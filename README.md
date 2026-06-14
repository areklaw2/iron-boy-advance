# Iron Boy Advance

Iron Boy Advance a Game Boy Advance emulator, written in rust.

## Features

- [ ] Game Boy Advance Components
  - [x] CPU (ARM7TDMI)
  - [x] Memory
  - [ ] Hardware
    - [x] LCD
    - [x] Sound
    - [x] Timers
    - [x] DMA Transfers
    - [ ] Communication Ports
      - [ ] Same computer Link Cable support
    - [x] Keypad
    - [x] Interrupts
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
    - [ ] Video Layer Toggling
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
  - [x] Game Controller input

## Testing

- [ARM7TDMI Single Step Test](https://github.com/SingleStepTests/ARM7TDMI) :white_check_mark:
- [gba-tests](https://github.com/jsmolka/gba-tests)
  - arm :white_check_mark:
  - thumb :white_check_mark:
  - bios :white_check_mark:
  - ppu :white_check_mark:
  - nes :white_check_mark:
  - memory :white_check_mark:
  - unsafe :white_check_mark:
- [arm-wrestler](https://github.com/destoer/armwrestler-gba-fixed) :white_check_mark:

## Acknowledgements and Sources

### Docs

- [GBATEK](https://problemkaputt.de/gbatek.htm) - The meat and potatoes, an amazing, essential peice of documention.
- [TONC](https://gbadev.net/tonc/foreword.html) - Helped alot with the component implementations and I had a lot of fun running the demos.

### Emulators

- [NanoBoyAdvance](https://github.com/nba-emu/NanoBoyAdvance) - An awesome project, it really helped me get a lot of the concepts of gba emulation, especially when I was stuck.
- [mgba](https://github.com/mgba-emu/mgba) - I've been a fan of this project since I knew what an emulator was; its so awesome.
- [rustboyadvance-ng](https://github.com/michelhe/rustboyadvance-ng) - the first emulator written in rust I ever came across and it inspired me to start making an eumlator.

### Test Suites

- [SingleStepTests/ARM7TDMI](https://github.com/SingleStepTests/ARM7TDMI) - this project was so helpful it gave me a great feedback loop when I was working on the CPU.
- [GBA Tests](https://github.com/jsmolka/gba-tests) - an essential test suite.
- [armwrestler-gba-fixed](https://github.com/destoer/armwrestler-gba-fixed) - 🦾 I was so happy when I got this to pass.

### Blogs

- [Gregory Gaines' blog](https://www.gregorygaines.com/) - Super helpful posts on the decomposing the ARM7TDMI instruction set and scheduler based game loops.
- [RadDad772's blog](https://raddad772.github.io/) - Great posts about the PPU that really help TONC click for me.
- [jsgroth's blog](https://jsgroth.dev/blog/) - Cool posts that helped me improve the sound quality of my emulator.

### Libraries

- [bitfields-rs](https://github.com/gregorygaines/bitfields-rs) - a crate created by Gregory Gaines (a really awesome dude) that help me streamline my code.

## License

Iron Boy Advance is licensed under the terms of the GNU General Public License (GPL) 3.0 or any later version. See [LICENSE](LICENSE) for details.

Copyright (C) 2026 Aza Walker
