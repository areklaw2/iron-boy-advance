#![allow(dead_code)]

use std::{env, fs, path::PathBuf};

use ironboyadvance::{Emulator, boot, detect_system, system_info};

const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
const FNV_PRIME: u64 = 1099511628211;

const LUMINANCE_RAMP: &[u8] = b"@%#*+=-:. ";

const TEST_UNIX_SECONDS: u64 = 1767225600;

pub fn repo_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(relative_path)
}

fn staged_rom_path(relative_path: &str) -> PathBuf {
    let source = repo_path(relative_path);
    let directory = env::temp_dir().join("ironboyadvance-tests");
    fs::create_dir_all(&directory).unwrap_or_else(|error| panic!("failed to create {directory:?}: {error}"));

    let staged = directory.join(relative_path.replace(['/', '\\', ' '], "_"));
    fs::copy(&source, &staged).unwrap_or_else(|error| panic!("failed to stage {source:?}: {error}"));
    let _ = fs::remove_file(staged.with_extension("sav"));

    staged
}

pub struct Headless {
    system: Box<dyn Emulator>,
    viewport_width: usize,
    viewport_height: usize,
    cycles_per_frame: usize,
    overshoot: usize,
}

impl Headless {
    pub fn load(relative_path: &str) -> Headless {
        let path = staged_rom_path(relative_path);
        let rom = fs::read(&path).unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
        let kind = detect_system(&rom).unwrap_or_else(|| panic!("unrecognized rom format: {path:?}"));
        let (viewport_width, viewport_height, _, _, cycles_per_frame) = system_info(kind);

        let system = boot(kind, &path.to_string_lossy(), rom, Vec::new(), TEST_UNIX_SECONDS, false)
            .unwrap_or_else(|error| panic!("failed to boot {path:?}: {error}"));

        Headless {
            system,
            viewport_width,
            viewport_height,
            cycles_per_frame,
            overshoot: 0,
        }
    }

    pub fn run_frame(&mut self) {
        self.overshoot = self.system.run(self.cycles_per_frame, self.overshoot);
        self.system.clear_audio_buffer();
    }

    pub fn run_frames(&mut self, frames: usize) {
        for _ in 0..frames {
            self.run_frame();
        }
    }

    pub fn frame_buffer(&self) -> &[u32] {
        self.system.frame_buffer()
    }

    pub fn read_memory(&self, address: u32) -> u8 {
        self.system.read_memory(address)
    }

    pub fn serial_text(&self) -> String {
        String::from_utf8_lossy(self.system.serial_output()).into_owned()
    }

    pub fn frame_hash(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        for pixel in self.system.frame_buffer() {
            for byte in pixel.to_le_bytes() {
                hash ^= byte as u64;
                hash = hash.wrapping_mul(FNV_PRIME);
            }
        }
        hash
    }

    pub fn frame_ascii(&self) -> String {
        let frame_buffer = self.system.frame_buffer();
        let mut rendered = String::new();

        for y in (0..self.viewport_height).step_by(2) {
            for x in 0..self.viewport_width {
                let pixel = frame_buffer[y * self.viewport_width + x];
                let red = (pixel >> 16) & 0xFF;
                let green = (pixel >> 8) & 0xFF;
                let blue = pixel & 0xFF;
                let luminance = (red * 54 + green * 183 + blue * 19) >> 8;
                let index = luminance as usize * (LUMINANCE_RAMP.len() - 1) / 0xFF;
                rendered.push(LUMINANCE_RAMP[index] as char);
            }
            rendered.push('\n');
        }

        rendered
    }
}
