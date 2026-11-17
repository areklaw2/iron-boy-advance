use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{CpuAction, barrel_shifter::ShiftType, memory::MemoryInterface, thumb::MoveShiftedRegister};
use ironboyadvance_common::memory::MemoryAccess;

use crate::{Compile, emit_action, emit_call, emit_epilogue, emit_prologue, guest};

impl Compile for MoveShiftedRegister {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let rd = self.rd() as u32;
        let rs = self.rs() as u32;
        let offset = self.offset() as u32;

        emit_prologue(assembler);
        dynasm! { assembler
            ; .arch aarch64
            ; mov x19, x0
            ; mov w1, #rs
        }
        emit_call(assembler, guest::register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w20, w0
            ; mov x0, x19
        }
        emit_call(assembler, guest::cpsr_carry::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w21, w0
        }

        dynasm! { assembler
            ; .arch aarch64
            ; mov w0, w20
            ; mov w1, #offset
            ; mov w2, w21
            ; mov w3, #1
        }
        match ShiftType::from(self.opcode()) {
            ShiftType::LSL => emit_call(assembler, guest::lsl as *const ()),
            ShiftType::LSR => emit_call(assembler, guest::lsr as *const ()),
            ShiftType::ASR => emit_call(assembler, guest::asr as *const ()),
            ShiftType::ROR => unimplemented!(),
        }
        dynasm! { assembler
            ; .arch aarch64
            ; mov w20, w0
            ; lsr x21, x0, #32
        }

        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
            ; mov w1, w20
        }
        emit_call(assembler, guest::set_cpsr_negative::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
            ; mov w1, w20
        }
        emit_call(assembler, guest::set_cpsr_zero::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
            ; mov w1, w21
        }
        emit_call(assembler, guest::set_cpsr_carry::<I> as *const ());

        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
            ; mov w1, #rd
            ; mov w2, w20
        }
        emit_call(assembler, guest::set_register::<I> as *const ());

        emit_action(
            assembler,
            CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential),
        );
        emit_epilogue(assembler);
    }

    fn ends_block(&self) -> bool {
        false
    }
}
