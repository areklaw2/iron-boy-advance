use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;

use crate::{
    io_registers::RegisterOps,
    ppu::registers::*,
    scheduler::event::{EventType, FutureEvent, InterruptEvent, PpuEvent},
};

const CYCLES_PER_PIXEL: usize = 4;

const HDRAW_PIXELS: usize = 240;
const HBLANK_PIXELS: usize = 68;
const HBLANK_FLAG_LAG: usize = 46;

pub const HDRAW_CYCLES: usize = HDRAW_PIXELS * CYCLES_PER_PIXEL + HBLANK_FLAG_LAG;
pub const HBLANK_CYCLES: usize = HBLANK_PIXELS * CYCLES_PER_PIXEL - HBLANK_FLAG_LAG;
const CYCLES_PER_SCANLINE: usize = HDRAW_CYCLES + HBLANK_CYCLES;

const VDRAW_SCANLINES: usize = 160;
const VBLANK_SCANLINES: usize = 68;
pub const VDRAW_CYCLES: usize = VDRAW_SCANLINES * CYCLES_PER_SCANLINE;
pub const VBLANK_CYCLES: usize = VBLANK_SCANLINES * CYCLES_PER_SCANLINE;

const MAX_V_COUNT: usize = VDRAW_SCANLINES + VBLANK_SCANLINES - 1;
pub const CYCLES_PER_FRAME: usize = VDRAW_CYCLES + VBLANK_CYCLES;

mod registers;

pub struct Ppu {
    lcd_control: LcdControl,
    green_swap: bool,
    lcd_status: LcdStatus,
    v_count: u8,
    bg_controls: [BgControl; 4],
    bg_x_offsets: [BgOffset; 4],
    bg_y_offsets: [BgOffset; 4],
    bg2_reference_points: [BgReferencePoint; 2],
    bg2_affine_parameters: [BgAffineParameter; 4],
    bg3_reference_points: [BgReferencePoint; 2],
    bg3_affine_parameters: [BgAffineParameter; 4],
    win_x_dimensions: [WindowDimension; 2],
    win_y_dimensions: [WindowDimension; 2],
    win_inside: WindowInside,
    win_outside: WindowOutside,
    mosiac_size: MosiacSize,
    color_special_effects_selection: ColorSpecialEffectsSelection,
    alpha_blending_coefficients: AlphaBlendingCoefficients,
    brightness_coefficient: BrightnessCoefficient,
    pallete_ram: Vec<u8>,
    vram: Vec<u8>,
    oam: Vec<u8>,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            lcd_control: LcdControl::from_bits(0),
            green_swap: false,
            lcd_status: LcdStatus::from_bits(0),
            v_count: 0,
            bg_controls: [BgControl::from_bits(0); 4],
            bg_x_offsets: [BgOffset::from_bits(0); 4],
            bg_y_offsets: [BgOffset::from_bits(0); 4],
            bg2_reference_points: [BgReferencePoint::from_bits(0); 2],
            bg2_affine_parameters: [BgAffineParameter::from_bits(0); 4],
            bg3_reference_points: [BgReferencePoint::from_bits(0); 2],
            bg3_affine_parameters: [BgAffineParameter::from_bits(0); 4],
            win_x_dimensions: [WindowDimension::from_bits(0); 2],
            win_y_dimensions: [WindowDimension::from_bits(0); 2],
            win_inside: WindowInside::from_bits(0),
            win_outside: WindowOutside::from_bits(0),
            mosiac_size: MosiacSize::from_bits(0),
            color_special_effects_selection: ColorSpecialEffectsSelection::from_bits(0),
            alpha_blending_coefficients: AlphaBlendingCoefficients::from_bits(0),
            brightness_coefficient: BrightnessCoefficient::from_bits(0),
            pallete_ram: vec![0; 0x400],
            vram: vec![0; 0x18000],
            oam: vec![0; 0x400],
        }
    }
}

impl SystemMemoryAccess for Ppu {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            // DISPCNT
            0x04000000..=0x04000001 => self.lcd_control.read_byte(address),
            // Green Swap
            0x04000002 => self.green_swap as u8,
            0x04000003 => 0,
            // DISPSTAT
            0x04000004..=0x04000005 => self.lcd_status.read_byte(address),
            // VCOUNT
            0x04000006..=0x04000007 => (self.v_count as u16).read_byte(address),
            // BG0CNT, BG1CNT, BG2CNT, BG3CNT
            0x04000008..=0x04000009 => self.bg_controls[0].read_byte(address),
            0x0400000A..=0x0400000B => self.bg_controls[1].read_byte(address),
            0x0400000C..=0x0400000D => self.bg_controls[2].read_byte(address),
            0x0400000E..=0x0400000F => self.bg_controls[3].read_byte(address),
            // BG0HOFS, BG0VOFS, BG1HOFS, BG1VOFS, BG2HOFS, BG2VOFS, BG3HOFS, BG3VOFS
            // BG2PA, BG2PB, BG2PC, BG2PD, BG2X_L, BG2X_H, BG2Y_L, BG2Y_H
            // BG3PA, BG3PB, BG3PC, BG3PD, BG3X_L, BG3X_H, BG3Y_L, BG3Y_H
            // WIN0H, WIN1H, WIN0V, WIN1V, WININ, WINOUT, MOSIAC
            0x04000010..=0x0400004F => 0,
            // BLDCNT, BLDALPHA, BLDY,
            0x04000050..=0x04000051 => self.color_special_effects_selection.read_byte(address),
            0x04000052..=0x04000053 => self.alpha_blending_coefficients.read_byte(address),
            0x04000054..=0x04000057 => self.brightness_coefficient.read_byte(address),
            // Access Memory
            0x05000000..=0x05FFFFFF => self.pallete_ram[(address & 0x3FF) as usize],
            0x06000000..=0x06FFFFFF => {
                let offset = (address & 0x1FFFF) as usize; // 128KB mirror
                let index = if offset >= 0x18000 { offset - 0x8000 } else { offset };
                self.vram[index]
            }
            0x07000000..=0x07FFFFFF => self.oam[(address & 0x3FF) as usize],
            _ => panic!("Invalid byte read for Ppu Register: {:#010X}", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            // DISPCNT
            0x04000000..=0x04000001 => self.lcd_control.write_byte(address, value),
            // Green Swap
            0x04000002 => self.green_swap = value & 0x1 != 0,
            0x04000003 => {}
            // DISPSTAT
            0x04000004..=0x04000005 => self.lcd_status.write_byte(address, value),
            // VCOUNT
            0x04000006..=0x04000007 => {}
            // BG0CNT, BG1CNT, BG2CNT, BG3CNT
            0x04000008..=0x04000009 => self.bg_controls[0].write_byte(address, value),
            0x0400000A..=0x0400000B => self.bg_controls[1].write_byte(address, value),
            0x0400000C..=0x0400000D => self.bg_controls[2].write_byte(address, value),
            0x0400000E..=0x0400000F => self.bg_controls[3].write_byte(address, value),
            // BG0HOFS, BG0VOFS, BG1HOFS, BG1VOFS, BG2HOFS, BG2VOFS, BG3HOFS, BG3VOFS
            0x04000010..=0x04000011 => self.bg_x_offsets[0].write_byte(address, value),
            0x04000012..=0x04000013 => self.bg_y_offsets[0].write_byte(address, value),
            0x04000014..=0x04000015 => self.bg_x_offsets[1].write_byte(address, value),
            0x04000016..=0x04000017 => self.bg_y_offsets[1].write_byte(address, value),
            0x04000018..=0x04000019 => self.bg_x_offsets[2].write_byte(address, value),
            0x0400001A..=0x0400001B => self.bg_y_offsets[2].write_byte(address, value),
            0x0400001C..=0x0400001D => self.bg_x_offsets[3].write_byte(address, value),
            0x0400001E..=0x0400001F => self.bg_y_offsets[3].write_byte(address, value),
            // BG2PA, BG2PB, BG2PC, BG2PD
            0x04000020..=0x04000021 => self.bg2_affine_parameters[0].write_byte(address, value),
            0x04000022..=0x04000023 => self.bg2_affine_parameters[1].write_byte(address, value),
            0x04000024..=0x04000025 => self.bg2_affine_parameters[2].write_byte(address, value),
            0x04000026..=0x04000027 => self.bg2_affine_parameters[3].write_byte(address, value),
            // BG2X_L, BG2X_H, BG2Y_L, BG2Y_H
            0x04000028..=0x0400002B => self.bg2_reference_points[0].write_byte(address, value),
            0x0400002C..=0x0400002F => self.bg2_reference_points[1].write_byte(address, value),
            // BG3PA, BG3PB, BG3PC, BG3PD
            0x04000030..=0x04000031 => self.bg3_affine_parameters[0].write_byte(address, value),
            0x04000032..=0x04000033 => self.bg3_affine_parameters[1].write_byte(address, value),
            0x04000034..=0x04000035 => self.bg3_affine_parameters[2].write_byte(address, value),
            0x04000036..=0x04000037 => self.bg3_affine_parameters[3].write_byte(address, value),
            // BG3X_L, BG3X_H, BG3Y_L, BG3Y_H
            0x04000038..=0x0400003B => self.bg3_reference_points[0].write_byte(address, value),
            0x0400003C..=0x0400003F => self.bg3_reference_points[1].write_byte(address, value),
            // WIN0H, WIN1H, WIN0V, WIN1V
            0x04000040..=0x04000041 => self.win_x_dimensions[0].write_byte(address, value),
            0x04000042..=0x04000043 => self.win_x_dimensions[1].write_byte(address, value),
            0x04000044..=0x04000045 => self.win_y_dimensions[0].write_byte(address, value),
            0x04000046..=0x04000047 => self.win_y_dimensions[1].write_byte(address, value),
            // WININ, WINOUT
            0x04000048..=0x04000049 => self.win_inside.write_byte(address, value),
            0x0400004A..=0x0400004B => self.win_outside.write_byte(address, value),
            // MOSIAC
            0x0400004C..=0x0400004F => self.mosiac_size.write_byte(address, value),
            // BLDCNT, BLDALPHA, BLDY,
            0x04000050..=0x04000051 => self.color_special_effects_selection.write_byte(address, value),
            0x04000052..=0x04000053 => self.alpha_blending_coefficients.write_byte(address, value),
            0x04000054..=0x04000057 => self.brightness_coefficient.write_byte(address, value),
            // Access Memory
            0x05000000..=0x05FFFFFF => self.pallete_ram[(address & 0x3FF) as usize] = value,
            0x06000000..=0x06FFFFFF => {
                let offset = (address & 0x1FFFF) as usize; // 128KB mirror
                let index = if offset >= 0x18000 { offset - 0x8000 } else { offset };
                self.vram[index] = value;
            }
            0x07000000..=0x07FFFFFF => self.oam[(address & 0x3FF) as usize] = value,
            _ => panic!("Invalid byte write for Ppu Register: {:#010X}", address),
        }
    }
}

impl Ppu {
    fn set_v_count(&mut self, value: u8) -> Option<InterruptEvent> {
        self.v_count = value;
        let is_match = self.lcd_status.v_count_setting() == self.v_count;
        self.lcd_status.set_v_counter_flag(is_match);
        match self.lcd_status.v_counter_irq_enable() && self.lcd_status.v_counter_flag() {
            true => Some(InterruptEvent::LcdVCounterMatch),
            false => None,
        }
    }

    pub fn handle_event(&mut self, event: PpuEvent) -> Vec<FutureEvent> {
        match event {
            PpuEvent::HDraw => self.handle_hdraw_complete(),
            PpuEvent::HBlank => self.handle_hblank_complete(),
            PpuEvent::VBlankHDraw => self.handle_vblank_hdraw_complete(),
            PpuEvent::VBlankHBlank => self.handle_vblank_hblank_complete(),
        }
    }

    fn handle_hdraw_complete(&mut self) -> Vec<FutureEvent> {
        let mut events = vec![];
        self.lcd_status.set_h_blank_flag(true);
        if self.lcd_status.h_blank_irq_enable() {
            events.push((EventType::Interrupt(InterruptEvent::LcdHBlank), 0));
        }
        events.push((EventType::Ppu(PpuEvent::HBlank), HBLANK_CYCLES));
        events
    }

    fn handle_hblank_complete(&mut self) -> Vec<FutureEvent> {
        let mut events = vec![];
        if let Some(v_count_match) = self.set_v_count(self.v_count + 1) {
            events.push((EventType::Interrupt(v_count_match), 0));
        }

        self.lcd_status.set_h_blank_flag(false);

        if (self.v_count as usize) < VDRAW_SCANLINES {
            self.render_scanline();
            events.push((EventType::Ppu(PpuEvent::HDraw), HDRAW_CYCLES));
        } else {
            self.lcd_status.set_v_blank_flag(true);
            if self.lcd_status.v_blank_irq_enable() {
                events.push((EventType::Interrupt(InterruptEvent::LcdVBlank), 0));
            }
            events.push((EventType::Ppu(PpuEvent::VBlankHDraw), HDRAW_CYCLES));
        }
        events
    }

    fn handle_vblank_hdraw_complete(&mut self) -> Vec<FutureEvent> {
        let mut events = vec![];
        self.lcd_status.set_h_blank_flag(true);
        if self.lcd_status.h_blank_irq_enable() {
            events.push((EventType::Interrupt(InterruptEvent::LcdHBlank), 0));
        }
        events.push((EventType::Ppu(PpuEvent::VBlankHBlank), HBLANK_CYCLES));
        events
    }

    fn handle_vblank_hblank_complete(&mut self) -> Vec<FutureEvent> {
        let mut events = vec![];
        self.lcd_status.set_h_blank_flag(false);

        if (self.v_count as usize) < MAX_V_COUNT {
            if let Some(v_count_match) = self.set_v_count(self.v_count + 1) {
                events.push((EventType::Interrupt(v_count_match), 0));
            }
            events.push((EventType::Ppu(PpuEvent::VBlankHDraw), HDRAW_CYCLES));
        } else {
            if let Some(v_count_match) = self.set_v_count(0) {
                events.push((EventType::Interrupt(v_count_match), 0));
            }
            self.lcd_status.set_v_blank_flag(false);
            self.render_scanline();
            events.push((EventType::Ppu(PpuEvent::HDraw), HDRAW_CYCLES));
        }
        events
    }

    fn render_scanline(&mut self) {
        todo!()
    }
}
