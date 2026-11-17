use ironboyadvance_arm7tdmi::barrel_shifter;

#[repr(C)]
pub struct ShiftResult {
    pub value: u32,
    pub carry: u32,
}

pub extern "C" fn lsl(value: u32, amount: u32, carry: u32) -> ShiftResult {
    let mut carry = carry != 0;
    let value = barrel_shifter::lsl(value, amount, &mut carry);
    ShiftResult {
        value,
        carry: carry as u32,
    }
}

pub extern "C" fn lsr(value: u32, amount: u32, carry: u32, immediate_shift: u32) -> ShiftResult {
    let mut carry = carry != 0;
    let value = barrel_shifter::lsr(value, amount, &mut carry, immediate_shift != 0);
    ShiftResult {
        value,
        carry: carry as u32,
    }
}

pub extern "C" fn asr(value: u32, amount: u32, carry: u32, immediate_shift: u32) -> ShiftResult {
    let mut carry = carry != 0;
    let value = barrel_shifter::asr(value, amount, &mut carry, immediate_shift != 0);
    ShiftResult {
        value,
        carry: carry as u32,
    }
}

pub extern "C" fn ror(value: u32, amount: u32, carry: u32, immediate_shift: u32) -> ShiftResult {
    let mut carry = carry != 0;
    let value = barrel_shifter::ror(value, amount, &mut carry, immediate_shift != 0);
    ShiftResult {
        value,
        carry: carry as u32,
    }
}
