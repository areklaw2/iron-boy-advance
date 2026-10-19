use crate::{Dissasemble, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::LoadStoreSignExtendedByteHalfword};

impl Dissasemble for LoadStoreSignExtendedByteHalfword {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let ro = self.ro();
        let rb = self.rb();
        let rd = self.rd();
        let signed = self.signed();
        let halfword = self.halfword();
        match (signed, halfword) {
            (false, false) => format!("STRH {}, [{},{}]", rd, rb, ro),
            (false, true) => format!("LDRH {}, [{},{}]", rd, rb, ro),
            (true, false) => format!("LDSB {}, [{},{}]", rd, rb, ro),
            (true, true) => format!("LDSH {}, [{},{}]", rd, rb, ro),
        }
    }
}
