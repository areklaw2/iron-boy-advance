# Iron Boy Advance

Iron Boy Advance a Game Boy Advance emulator, written in rust.

## Features

- [ ] Game Boy Advance Components
  - [x] CPU (ARM7TDMI)
  - [ ] Memory
    - [ ] Open Bus
  - [ ] Hardware
    - [ ] LCD
    - [ ] Sound
    - [ ] Timers
    - [ ] DMA Transfers
    - [ ] Communication Ports
      - [ ] Same computer Link Cable support
    - [ ] Keypad
    - [ ] Interrupts
    - [ ] System Control
    - [ ] Cartridges
      - [ ] Real-time clock support
    - [ ] Bios
      - [ ] Ability to load External BIOS
- [ ] Game Boy/ Game Boy Color support
- [ ] Just-in-time (JIT) compilation
- [ ] Scheduler based game Loop
- [ ] UI
  - [ ] Desktop frontend
    - [ ] Graphics Views
    - [ ] Palette Viewer
    - [ ] Sprite Viewer
    - [ ] Tile Viewer
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

## Testing

- ARM7TDMI Single Step Test :white_check_mark:
- gba-tests
  - arm :white_check_mark:
