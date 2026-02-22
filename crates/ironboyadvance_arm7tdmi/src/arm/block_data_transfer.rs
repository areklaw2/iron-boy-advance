use crate::BitOps;

use crate::{
    CpuAction, CpuMode, Register,
    arm::arm_instruction,
    cpu::{Arm7tdmiCpu, PC},
    memory::{MemoryAccess, MemoryInterface},
};

#[derive(Debug, Clone, Copy)]
pub struct BlockDataTransfer {
    value: u32,
}

arm_instruction!(BlockDataTransfer);

impl BlockDataTransfer {
    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let mut register_list = self.register_list();
        let rn = self.rn() as usize;
        let mut address = cpu.register(rn);

        let mut transfer_pc = register_list.contains(&PC);
        let transfer_bytes = if !register_list.is_empty() {
            register_list.len() as u32 * 4
        } else {
            register_list.push(PC);
            transfer_pc = true;
            64
        };

        let load = self.load();
        let load_psr_force_user = self.load_psr_force_user();
        let mode = cpu.cpsr().mode();
        let switch_mode =
            load_psr_force_user && (!load || !transfer_pc) && ![CpuMode::User, CpuMode::System].contains(&mode);
        if switch_mode {
            cpu.cpsr_mut().set_mode(CpuMode::User);
        }

        let add = self.add();
        let mut pre_index = self.pre_index();
        let mut base_address = address;
        if !add {
            pre_index = !pre_index;
            address -= transfer_bytes;
            base_address -= transfer_bytes;
        } else {
            base_address += transfer_bytes
        }

        let write_back = self.write_back();
        let mut memory_access = MemoryAccess::NonSequential;
        let mut action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);
        match load {
            true => {
                for (i, register) in register_list.iter().enumerate() {
                    if pre_index {
                        address += 4
                    }

                    let value = cpu.load_32(address, memory_access as u8);
                    if write_back && i == 0 {
                        if rn == PC {
                            base_address += 4;
                            if !transfer_pc {
                                cpu.pipeline_flush();
                            }
                        }
                        cpu.set_register(rn, base_address);
                    }
                    cpu.set_register(*register, value);

                    if !pre_index {
                        address += 4
                    }

                    memory_access = MemoryAccess::Sequential;
                }

                cpu.idle_cycle();
                if transfer_pc {
                    if load_psr_force_user {
                        cpu.set_cpsr(cpu.spsr());
                    }

                    cpu.pipeline_flush();
                    action = CpuAction::PipelineFlush;
                }
            }
            false => {
                for (i, register) in register_list.iter().enumerate() {
                    if pre_index {
                        address += 4
                    }

                    let mut value = cpu.register(*register);
                    if *register == PC {
                        match write_back && rn == PC {
                            true => value -= 4,
                            false => value += 4,
                        }
                    }

                    cpu.store_32(address, value, memory_access as u8);
                    if write_back && i == 0 {
                        if rn == PC {
                            base_address += 4;
                            cpu.pipeline_flush();
                        }
                        cpu.set_register(rn, base_address);
                    }

                    if !pre_index {
                        address += 4
                    }

                    memory_access = MemoryAccess::Sequential;
                }
            }
        }

        if switch_mode {
            cpu.cpsr_mut().set_mode(mode);
        }

        action
    }

    pub fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let pre_index = self.pre_index();
        let add = self.add();
        let load_psr_force_user = if self.load_psr_force_user() { "^" } else { "" };
        let write_back = if self.write_back() { "!" } else { "" };
        let load = self.load();
        let rn = self.rn();
        let register_list = self
            .register_list()
            .iter()
            .map(|register| Register::from(*register as u32).to_string())
            .collect::<Vec<String>>()
            .join(",");

        let mnemonic = match (load, pre_index, add) {
            (true, true, true) => match rn == Register::R13 {
                true => format!("LDM{}ED", cond),
                false => format!("LDM{}IB", cond),
            },
            (true, false, true) => match rn == Register::R13 {
                true => format!("LDM{}FD", cond),
                false => format!("LDM{}IA", cond),
            },
            (true, true, false) => match rn == Register::R13 {
                true => format!("LDM{}EA", cond),
                false => format!("LDM{}DB", cond),
            },
            (true, false, false) => match rn == Register::R13 {
                true => format!("LDM{}FA", cond),
                false => format!("LDM{}DA", cond),
            },
            (false, true, true) => match rn == Register::R13 {
                true => format!("STM{}FA", cond),
                false => format!("STM{}IB", cond),
            },
            (false, false, true) => match rn == Register::R13 {
                true => format!("STM{}EA", cond),
                false => format!("STM{}IA", cond),
            },
            (false, true, false) => match rn == Register::R13 {
                true => format!("STM{}FD", cond),
                false => format!("STM{}DB", cond),
            },
            (false, false, false) => match rn == Register::R13 {
                true => format!("STM{}ED", cond),
                false => format!("STM{}DA", cond),
            },
        };

        format!("{} {}{},({}){}", mnemonic, rn, write_back, register_list, load_psr_force_user)
    }

    #[inline]
    pub fn rn(&self) -> Register {
        self.value.bits(16..=19).into()
    }

    #[inline]
    pub fn pre_index(&self) -> bool {
        self.value.bit(24)
    }

    #[inline]
    pub fn add(&self) -> bool {
        self.value.bit(23)
    }

    #[inline]
    pub fn write_back(&self) -> bool {
        self.value.bit(21)
    }

    #[inline]
    pub fn load(&self) -> bool {
        self.value.bit(20)
    }

    #[inline]
    pub fn load_psr_force_user(&self) -> bool {
        self.value.bit(22)
    }

    #[inline]
    pub fn register_list(&self) -> Vec<usize> {
        (0..=15).filter(|&i| self.value.bit(i)).collect()
    }
}
