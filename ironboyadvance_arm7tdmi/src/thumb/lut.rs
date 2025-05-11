use super::ThumbInstructionKind;

pub fn generate_thumb_lut() -> [ThumbInstructionKind; 1024] {
    let mut arm_lut = [ThumbInstructionKind::Undefined; 1024];
    for i in 0..1024 {
        arm_lut[i] = decode_arm((i as u16) << 6);
    }
    arm_lut
}

fn decode_arm(instruction: u16) -> ThumbInstructionKind {
    todo!()
}
