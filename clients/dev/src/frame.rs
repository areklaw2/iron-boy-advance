use getset::CopyGetters;
use ironboyadvance_core::FPS;

const FRAME_DURATION_NANOS: f32 = 1_000_000_000.0 / FPS;
const FRAME_DURATION: std::time::Duration = std::time::Duration::from_nanos(FRAME_DURATION_NANOS as u64);


#[derive(CopyGetters)]
pub struct FrameTimer {
    frame_count: u16,
    frame_clock: std::time::Instant,
    fps_clock: std::time::Instant,
    #[getset(get_copy = "pub")]
    fps: f64,
}

impl FrameTimer {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            frame_clock: std::time::Instant::now(),
            fps_clock: std::time::Instant::now(),
            fps: 0.0,
        }
    }

    pub fn slow_frame(&mut self) {
        let target = self.frame_clock + FRAME_DURATION;
        if let Some(remaining) = target.checked_duration_since(std::time::Instant::now()) {
            std::thread::sleep(remaining);
        }
        self.frame_clock = target;
    }

    pub fn count_frame(&mut self) {
        self.frame_count += 1;
        let time_elapsed = self.fps_clock.elapsed();
        if time_elapsed.as_secs() >= 1 {
            self.fps = self.frame_count as f64 / time_elapsed.as_secs_f64();
            self.frame_count = 0;
            self.fps_clock = std::time::Instant::now();
        }
    }
}
