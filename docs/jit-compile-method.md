# Writing a JIT `compile()` Method

## 1. The Mental Model

Compiled blocks are native ARM64 code in an mmap'd (memory mapped) buffer. They can't touch the `Arm7tdmiCpu` struct directly, because they know nothing about Rust's types or memory layout.

Think of a block as a list of phone calls: "dial this number, say these words, hang up, dial the next." The block never does the work itself; whoever answers does. Each person answering is a **guest helper**: a plain Rust function, marked `extern "C"`, that does one piece of guest work (`set_register`, `pipeline_flush`). At JIT-compile time you take a helper's address as a number, write that number into the emitted code, and emit a call to it. Helpers live in `guest.rs`, so call sites read `guest::set_register`.

The emitted block is then effectively "load arguments, jump to Rust, come back", over and over. All semantics stay in Rust. The ARM64 only marshals arguments and sequences calls.

## 2. ARM64, the ABI, and dynasm

### The ARM64 You Need

ARM64 has 31 general registers `x0`-`x30`, 64-bit. Each has a 32-bit alias `w0`-`w30`, the same physical register, and **writing a `w` register zeroes the top 32 bits**. Guest ARM7TDMI registers are 32-bit, so use `w` for guest values and `x` for pointers.

`xzr`/`wzr` reads as 0 and discards writes. `sp` is the stack pointer. `x30` is the link register.

#### Moving Data

```as
mov x0, x19                   // copy register to register
movz x9, #0x1234              // "move zero": x9 = 0x1234, all other bits ZEROED
movk x9, #0xABCD, lsl #16     // "move keep": overwrite bits 16..31 only
movn w0, #0                   // "move NOT":  w0 = !0 = 0xFFFFFFFF
```

`movn` writes the complement of its immediate, so `0xFFFFFFFF` costs one instruction instead of a pair. You use it for the `PipelineFlush` return.

#### Memory

```as
ldr w10, [x19, #60]         // load 32 bits from (x19 + 60)
str w10, [x19, #56]         // store w10 to (x19 + 56)
stp x19, x30, [sp, #-16]!   // store pair, PRE-index
ldp x19, x30, [sp], #16     // load pair, POST-index
```

- `[sp, #-16]!` is pre-index: `sp = sp - 16` first, _then_ store. The `!` writes the new `sp` back. This is a push.
- `[sp], #16` is post-index: load first, _then_ `sp = sp + 16`. Pop.

One `stp` push plus one `ldp` pop is 16 bytes, keeping the stack 16-byte aligned. That's why the idiom is a pair, not two separate pushes.

#### Control Flow

```as
blr x9    // call: x30 = address of NEXT instruction, then jump to x9
ret       // return: jump to x30
b   label / b.eq label / cbz w0, label / cbnz w0, label
```

`blr` clobbers `x30`. `ret` is literally "jump to whatever is in x30". This is the most important fact in the ABI: `blr` without saving `x30` first destroys your own return address, and `ret` jumps back into your own block instead of to your caller.

#### Arithmetic

```as
add w0, w1, w2       // w0 = w1 + w2
add w0, w1, #4       // immediate is only 12 bits
sub w0, w1, #4
mul w0, w1, w2
orr / and / eor / bic  // bitwise (bic = a & !b)
lsl / lsr / asr      // shifts (asr = arithmetic, sign-preserving)
adds / subs          // same but SET the NZCV flags
cmp w0, #16          // subs that throws the result away, keeps flags
```

### The Five ABI Rules

The ABI is a contract about who owns which register. rustc-compiled functions obey it; your emitted code is a caller, so it must as well or rustc's code corrupts your state.

1. **Arguments** go in `x0`-`x7`, left to right. Return value in `x0`. A 32-bit argument uses the `w` half. So `set_register(cpu, reg, value)` means `x0` = cpu, `w1` = reg, `w2` = value.
2. **Caller-saved: `x0`-`x18`.** The callee may destroy any of these. If you have something in `x9` and you call, `x9` is garbage afterward. Corollary: `x9`-`x15` are free scratch _between_ calls, which is why `emit_call` uses `x9`.
3. **Callee-saved: `x19`-`x28`.** The callee gives these back unchanged. That's why you park the cpu pointer in `x19`. The promise runs both ways: you're a callee too, so you save and restore them yourself.
4. **`x30` is the link register.** Any call overwrites it. Save it if you call anything. (`x29` is the frame pointer; ignore it, you're not producing unwindable frames.)
5. **`sp` must be 16-byte aligned** wherever it's used as a base address, not just at call boundaries. On Apple Silicon a misaligned `sp` faults. Always move it in multiples of 16.

These five rules will carry the whole JIT implementation. The AAPCS64 spec is about 100 pages; the rest covers floating point, varargs, structs by value, and unwinding, none of which your blocks do.

### dynasm Macro Syntax

None of these are guessable:

- **Every instruction line starts with `;`.** That's dynasm syntax, not a comment. A line without it is a Rust statement.
- **`#` introduces an immediate**, and everything after it parses as a Rust expression: `#14`, `#byte_0`, `#(value & 0xFFFF)` all work. Prefer a bare `let` binding. Inline parens trip `unused_parens` on dynasm's own expansion, one warning per line, unfixable from the call site.
- **`W(expr)` / `X(expr)`** are dynamic register selectors; the register number is a runtime `u8` (not `u32`, or you get a confusing `From<u32>` error).
- **`.arch aarch64` is per-invocation.** dynasm's default is the _host_ architecture, so on a Mac it's redundant. Keep it anyway; it costs nothing and the block stops being correct-by-accident.
- **One sequence can span many `dynasm!` blocks.** They append to the same assembler and the bytes stay contiguous. That's what lets you alternate `dynasm!` blocks with `emit_call` mid-sequence.

---

## 3. The Per-Instruction Loop

The loop per instruction:

1. **Split** the interpreter's `execute` body into compile-time and runtime halves.
2. **Probe**: write the runtime half as a flat `extern "C"` function, with compile-time values as literals.
3. `just probe <instruction>` and read the listing.
4. **Translate** the listing into `compile()`.
5. Ask of every instruction you copied: does it survive a different constant?
6. **Test** against SingleStepTests. The interpreter is the source of truth.

### Compile-Time vs. Runtime Decisions

`compile()` runs with the decoded instruction in hand, before the block executes. Anything decoded from the instruction word is known _then_: register numbers, the S bit, the accumulate bit, immediates, the PSR field mask, whether `rd == PC`.

Anything that depends on CPU _state_ is only known when the block runs: register contents, the current mode, flags.

An example:

```rust
// interpreter
let mut value = cpu.register(rm);
if rm == PC { value += 4; }          // rm is from the instruction word
if cpu.cpsr().mode() == User { ... } // mode is CPU state
```

The first `if` becomes a plain Rust `if` in `compile()` that decides whether to **emit** an `add`. The second becomes an **emitted** `cmp` + branch, because nobody knows the mode until the block runs. Mixing these up is the most common design mistake: either you emit a runtime check for something already known, or you try to decide at compile time something that isn't.

---

## 4. Probing

The probe is a plain Rust file of `extern "C"` blocks. rustc compiles it, and you read the assembly rustc wrote. You're borrowing a compiler that already knows the ABI, instead of guessing.

```rust
unsafe extern "C" {
    fn guest_register(cpu: *mut u8, register: u32) -> u32;
    fn guest_set_register(cpu: *mut u8, register: u32, value: u32);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_1(cpu: *mut u8) -> u32 {
    unsafe {
        let value = guest_register(cpu, 1) + 4;
        guest_set_register(cpu, 0, value);
    }
    0b11
}
```

### Rules for Good Probes

- **One probe per shape, not per value.** `LSL #3` and `LSL #7` are the same probe. `LSL` and `LSR #0` are not, because they emit different instructions. MUL and MLA are two shapes (accumulate adds a fetch, an `add` and an idle cycle). So are MSR to CPSR and MSR to SPSR.
- **One probe can cover every option.** Turn all the decode-time options on (PC adjust, S bit, `rd == PC` flush). Each `if` in `compile()` then keeps or drops a chunk of the listing.
- **Copy the real helper's signature exactly.** Arguments go in registers by position. Drop or reorder one and the listing shows the wrong registers.
- **For compile-time inputs, write the real `match`.** rustc folds it down to the one arm that applies, so the listing is your answer.
- **Don't write Rust that panics.** `value >> 32` panics in debug. Write that case by hand.
- **A branch that survives in the listing is real.** If rustc can't fold a `match` on a runtime value, your emitted code needs that branch too. Ask whether it's worth inlining or should stay in one helper.
- **A call in the listing means one of two things.** Either the value is only known at runtime, so you need a branch. Or the operation needs CPU state that isn't in a host register yet (RRX's carry-in), so you need a call. The branch stays forever. The call goes away once you cache that state.
- **A helper's return value can feed straight into native math.** `blr`, then use `w0` in the next instruction.

### The Probe Is a Reference, Not Gospel

rustc translates your toy source literally. If the probe source calls `guest_cpsr_mode(cpu)` twice in one `if`, the listing calls it twice, but nothing clobbers `w0` between two `cmp`s, so one call is enough in your `compile()`. Likewise rustc may pick `csel` (compute both arms, select one) where an honest `cmp` + branch is easier to write by hand. Copy the _shape_, then simplify.

And when rustc proves a branch is dead, it deletes it. A probe where both arms of a mode check produce the same value shows no `cmp` at all. That's a valid optimization, but it's yours to make deliberately in `compile()`, not to inherit by accident.

---

## 5. Writing `compile()`

### The Register Map

Think of the callee-saved registers as labeled shelves: anything you need _after_ a call goes on a shelf, because the counter (`x0`-`x18`) gets wiped every time you call someone.

| register | holds                                                |
| -------- | ---------------------------------------------------- |
| `x19`    | the cpu pointer, for the whole block                 |
| `w20`    | the main working value (operand, product, result)    |
| `w21`    | a second value that must survive a call (a mask, rs) |
| `w22`    | a third (a user-mode mask, an accumulator)           |
| `x9`     | **never** for anything live: `emit_call` uses it     |

The prologue saves the caller's `x19`-`x22` and `x30`; the epilogue puts them back right before `ret`. In between, the shelves are yours. The epilogue doesn't "clear" them, it restores what the caller had.

```rust
fn emit_prologue(assembler: &mut Assembler<Aarch64Relocation>) {
    dynasm! { assembler
        ; .arch aarch64
        ; stp x22, x21, [sp, #-48]!
        ; stp x20, x19, [sp, #16]
        ; str x30, [sp, #32]
    }
}
```

### `mov x0, x19` Before Every Call

`emit_call` only loads the target address into `x9` and does `blr x9`. It does **not** set up arguments. After any call, `x0` holds that call's _return value_, not the cpu pointer. So every call needs:

```as
; mov x0, x19      // cpu pointer back into arg 0
; mov w1, ...      // other args
  emit_call(...)
```

Get this wrong and the next helper dereferences a guest value as a pointer. That shows up as a panic like `misaligned pointer dereference: address must be a multiple of 0x8 but is 0xe0000052`. Here `0xe0000052` is a CPSR value (N, Z, C set, IRQ mode) being used as `cpu`. If the bad address looks like guest data, look for a missing `mov x0, x19`, or a `mov x19, x0` written backwards.

### Move the Result Home Immediately

After a helper returns, move `w0` onto a shelf **unconditionally**, then adjust:

```rust
dynasm! { assembler
    ; .arch aarch64
    ; mov x0, x19
    ; mov w1, #rm
  }
emit_call(assembler, guest::register::<I> as *const ());
dynasm! { assembler
    ; .arch aarch64
    ; mov w20, w0
}
if rm as usize == PC {
    dynasm! { assembler
        ; .arch aarch64
        ; add w20, w20, #4
    }
}
```

A common bug is moving only inside the `if`: the PC case works, and every other register reads garbage.

### Return Codes

The block's return value tells `run()` what happened. It's the interpreter's `CpuAction` encoded as a number, where `MemoryAccess` is `NonSequential = 0b0`, `Sequential = 0b1`, `Instruction = 0b10`:

| interpreter returns                     | emit          |
| --------------------------------------- | ------------- |
| `Advance(Instruction \| Sequential)`    | `mov w0, #3`  |
| `Advance(Instruction \| NonSequential)` | `mov w0, #2`  |
| `PipelineFlush`                         | `movn w0, #0` |

Don't guess the code. Read what the interpreter's `execute` returns for that instruction and encode that. Instructions with internal cycles (shift by register, multiply) return NonSequential, so `#2`.

### The `rd == PC` Tail

Writing `rd` and potentially flushing is the same at the end of many instructions. This is Multiply's version:

```rust
match rd as usize == PC {
    true => {
        dynasm! { assembler ; .arch aarch64 ; mov x0, x19 }
        emit_call(assembler, guest::pipeline_flush::<I> as *const ());
        dynasm! { assembler ; .arch aarch64 ; movn w0, #0 }
    }
    false => dynasm! { assembler ; .arch aarch64 ; mov w0, #2 },
}
emit_epilogue(assembler);
```

The `#2` is Multiply's return code (internal cycles make it NonSequential). Take the non-flush code from the return-code table above, per instruction: DataProcessing emits `#3`, or `#2` for shift-by-register.

### Labels

A label marks a spot in the emitted code so a branch can jump to it. You need one only for a decision made at runtime, like the CPU mode. Check that first:

- Condition codes are handled in `cycle()`.
- `rd == PC` and register lists come from the instruction word, so they're compile-time `if`s.
- A runtime check that's emulator plumbing rather than guest behavior belongs in one helper.

```rust
use dynasmrt::DynasmLabelApi;

; cmp w0, #16
; b.eq >user       // ">" = jump FORWARD to the next "user:"
...
; b >done
; user:            // definition: emits zero bytes
...
; done:
```

There are three kinds. Use local labels unless you have a reason not to:

- **Local** (`>name` forward, `<name` backward, `name:` to define). Every `compile()` can reuse the same name, and each reference binds to the nearest matching definition.
- **Dynamic** (`let label = assembler.new_dynamic_label();`, then `=>label` to reference and define). The ID lives in a variable, so use it when a label must be passed around or outlive one instruction, as block linking will.
- **Global** (`->name`). Avoid. The name must be unique across the whole assembler, so the second instruction that uses it panics.

An undefined label (`>skip` with no `skip:`) isn't a compile error. It fails at `commit()`, when the block runs, so `cargo check` won't catch it.

To test a Rust `bool` return, use `tbnz w0, #0`, not `cbnz w0`. AAPCS64 leaves the high bits of narrow returns unspecified, so only bit 0 is guaranteed.

### Merging Under a Mask

`MSR` overwrites only some bits of the status register, the ones the field mask selects. Think of the mask as a stencil laid over the old value: you paint the new bits only through the holes. With the old value in `w0`, the new value in `w20` and the mask in `w21`:

```as
; bic w0, w0, w21    // keep the bits OUTSIDE the mask
; and w1, w20, w21   // take the bits INSIDE the mask
; orr w1, w0, w1     // combine
```

The mask goes in a register, not into `and w0, w0, #mask`. ARM64 can only encode certain repeating bit patterns as an immediate in `and`/`orr` (`0x12345678` and `0` don't fit), and a mask built from instruction fields can be anything. Load it with `emit_immediate_32(assembler, register, value)`, which is a `movz` for the low 16 bits and a `movk` for the high 16.

### Writing Helpers

Helpers live in `guest.rs`. Conventions:

- Every helper takes `cpu: *mut Arm7tdmiCpu<I>` first and is two lines: unwrap the pointer, delegate to the real method. Explicit beats clever at an `extern "C"` boundary; don't macro them away.
- **Take `u32`, convert inside.** Assembly has no bool. Accept `value: u32` and do `!= 0`, `>> 31`, or `== 0` in Rust, so the block just passes the whole result:

  ```rust
  pub unsafe extern "C" fn set_cpsr_negative<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, value: u32) {
      let cpu = unsafe { &mut *cpu };
      cpu.cpsr_mut().set_negative(value >> 31 != 0);
  }
  ```

  When copy-pasting a helper to make a sibling, change the method it calls. `set_cpsr_zero` calling `set_negative` compiles fine and shows up as N and Z swapped in the test diff.

- **Push runtime-variable work into one helper.** A cycle count that depends on an operand (multiply's 1-4 idle cycles) is one call to a helper that loops host-side, instead of an emitted compare chain.
- **Two return values: a `#[repr(C)]` struct of two `u32`s** comes back packed in `x0` (first field low, second high), so unpack the second with `lsr x_, x0, #32`. It's the one return that isn't just `w0`.

---

## 6. Debugging

### Look at What You Emitted

Tests tell you whether what you meant was right. Disassembly tells you whether you emitted what you meant. You can skip it while blocks are mostly helper calls: a mistake there blows up inside a Rust helper, and Rust gives you a readable panic. Reach for it when you get a bare segfault with no message, or once you inline instructions and there's no Rust left to panic for you.

**1. Dump one block.** Compile one specific instruction in isolation, from its hex encoding:

```bash
just dump arm 0xE0234291
just dump thumb 0x4348
```

This writes the bytes to `/tmp/block.bin` and disassembles them. Pick an encoding that exercises the shape you care about (here, MLA with `rd = 3`). It runs outside `cycle()` and the test suite on purpose: thousands of single-step cases would each overwrite the file. `compile()` never touches the bus, so the tool uses a stand-in one.

**2. Count first.** The file is nothing but 4-byte instructions, so its size is the instruction count:

```bash
echo $(( $(wc -c < /tmp/block.bin) / 4 ))
```

Predict the number from your probe listing before you look. Each `bl` becomes five instructions (`emit_call`'s four `movz`/`movk` plus `blr`), and the prologue skips rustc's frame-pointer setup. If the count is off, you emitted something you didn't mean to, and you found out without reading anything.

**3. Read the disassembly.** The dump is a bare strip of bytes with no header saying "this is ARM64, start here", and macOS `objdump` has no option to override that. `just dump` handles it by wrapping the bytes in a header with rustup's `llvm-tools` and disassembling. Add `--raw` for the hex encodings next to the mnemonics, or re-run on the existing file with `just disasm`:

```bash
just dump arm 0xE0234291 --raw
just disasm
just disasm path/to/other.bin
```

The `movz`/`movk` values will be your real helper addresses. Everything else should match your probe line for line. Offsets step by 4; anything else means non-instruction bytes got into the stream.

Aliases make the output look different from your source even when the bytes are identical. Don't chase them:

| you emitted        | disassembly shows |
| ------------------ | ----------------- |
| `movz x9, #0x5678` | `mov x9, #0x5678` |
| `movn w0, #0`      | `mov w0, #-0x1`   |
| `orr x0, xzr, x1`  | `mov x0, x1`      |

### Symptoms

| symptom                                     | look at                                                |
| ------------------------------------------- | ------------------------------------------------------ |
| misaligned pointer, address looks like data | a call without `mov x0, x19`                           |
| segfault, instructions look right           | ABI: `x30` saved? every `x19`-`x28` you touched saved? |
| `DynasmError` at `commit()`                 | a label referenced but never defined                   |
| flags wrong, value right                    | a helper calling the wrong setter; the S-bit `match`   |
| non-PC registers wrong, PC case fine        | the result moved home only inside the PC `if`          |

**Reading test failures.** The single-step assert compares the full CPSR as a number. Convert both sides to hex and compare the top nibble (NZCV) and the bottom byte (mode, T, I, F) to see which bits flipped. The run stops at the first failing case, so fix it and rerun; there may be more behind it.

---

## 7. Things That Will Bite Before the Tests Do

- `cycle()` compiles fresh every execution and never caches. Correct but pointless as a JIT. Fine for now: semantics first, caching later.
- The assembler buffer grows forever. `cycle` appends a block per instruction and never reclaims, so watch memory until the cache exists.
- Condition codes are checked in Rust in `cycle` before compiling. Correct and simpler than emitting `b.cond`. Leave it.
- `pc()` inside a block returns the _pipeline-adjusted_ pc, the same as the interpreter sees. Don't "fix" it.
- Don't apply optimizations a probe hints at until you choose to. Get the straightforward, always-branch version passing first.
