use crate::{DesktopError, app::RunningContent, video::fit_scale};

const SCREENSHOT_DIRECTORY: &str = "screenshots";

pub fn save_screenshot(running: &RunningContent, window_width: u32, window_height: u32) -> Result<(), DesktopError> {
    //TODO: when a screenshot is taken we need to give some type of overlay or visual notification that it happened

    let Some(frame_buffer) = &running.last_frame else {
        tracing::warn!("screenshot requested before any frame has arrived");
        return Ok(());
    };

    let src_w = running.emulator.viewport_width as u32;
    let src_h = running.emulator.viewport_height as u32;

    let scale = fit_scale(
        window_width,
        window_height,
        running.emulator.viewport_width,
        running.emulator.viewport_height,
    );
    let out_w = (src_w as f32 * scale).round().max(1.0) as u32;
    let out_h = (src_h as f32 * scale).round().max(1.0) as u32;

    let mut rgba_buffer = Vec::with_capacity((out_w as usize) * (out_h as usize) * 4);
    for y in 0..out_h {
        let src_y = (((y as f32) / scale) as u32).min(src_h - 1);
        for x in 0..out_w {
            let src_x = (((x as f32) / scale) as u32).min(src_w - 1);
            let pixel = frame_buffer[(src_y * src_w + src_x) as usize];
            rgba_buffer.push(((pixel >> 16) & 0xFF) as u8);
            rgba_buffer.push(((pixel >> 8) & 0xFF) as u8);
            rgba_buffer.push((pixel & 0xFF) as u8);
            rgba_buffer.push(0xFF);
        }
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let name = format!("{SCREENSHOT_DIRECTORY}/screenshot-{timestamp}.png");

    std::fs::create_dir_all(SCREENSHOT_DIRECTORY)?;
    image::save_buffer(&name, &rgba_buffer, out_w, out_h, image::ColorType::Rgba8)?;
    tracing::info!("wrote {name} ({out_w}x{out_h})");
    Ok(())
}
