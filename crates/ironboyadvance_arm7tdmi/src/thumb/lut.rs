use ThumbInstructionKind::*;

use super::ThumbInstructionKind;

pub fn generate_thumb_lut() -> [ThumbInstructionKind; 1024] {
    let mut thumb_lut = [Undefined; 1024];
    for i in 0..1024 {
        thumb_lut[i] = decode_thumb((i as u16) << 6);
    }
    thumb_lut
}

fn decode_thumb(instruction: u16) -> ThumbInstructionKind {
    if instruction & 0xF800 < 0x1800 {
        MoveShiftedRegister
    } else if instruction & 0xF800 == 0x1800 {
        AddSubtract
    } else if instruction & 0xE000 == 0x2000 {
        MoveCompareAddSubtractImmediate
    } else if instruction & 0xFC00 == 0x4000 {
        AluOperations
    } else if instruction & 0xFC00 == 0x4400 {
        HiRegisterOperationsBranchExchange
    } else if instruction & 0xF800 == 0x4800 {
        PcRelativeLoad
    } else if instruction & 0xF200 == 0x5000 {
        LoadStoreRegisterOffset
    } else if instruction & 0xF200 == 0x5200 {
        LoadStoreSignExtendedByteHalfword
    } else if instruction & 0xE000 == 0x6000 {
        LoadStoreImmediateOffset
    } else if instruction & 0xF000 == 0x8000 {
        LoadStoreHalfword
    } else if instruction & 0xF000 == 0x9000 {
        SpRelativeLoadStore
    } else if instruction & 0xF000 == 0xA000 {
        LoadAddress
    } else if instruction & 0xFF00 == 0xB000 {
        AddOffsetToSp
    } else if instruction & 0xF600 == 0xB400 {
        PushPopRegisters
    } else if instruction & 0xF000 == 0xC000 {
        MultipleLoadStore
    } else if instruction & 0xFF00 < 0xDF00 {
        ConditionalBranch
    } else if instruction & 0xFF00 == 0xDF00 {
        SoftwareInterrupt
    } else if instruction & 0xF800 == 0xE000 {
        UnconditionalBranch
    } else if instruction & 0xF000 == 0xF000 {
        LongBranchWithLink
    } else {
        Undefined
    }
}
