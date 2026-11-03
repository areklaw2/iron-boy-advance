use ironboyadvance::Viewport;
use winit::dpi::PhysicalSize;

use crate::{DesktopError, video::frame_renderer::fit_scale};

const SCREENSHOT_DIRECTORY: &str = "screenshots";

pub(crate) fn save_screenshot(
    frame: &[u32],
    viewport: Viewport,
    window_size: PhysicalSize<u32>,
) -> Result<(), DesktopError> {
    //TODO: when a screenshot is taken we need to give some type of overlay or visual notification that it happened

    let src_w = viewport.width as u32;
    let src_h = viewport.height as u32;

    let scale = fit_scale(window_size.width, window_size.height, viewport);
    let out_w = (src_w as f32 * scale).round().max(1.0) as u32;
    let out_h = (src_h as f32 * scale).round().max(1.0) as u32;

    let mut rgba_buffer = Vec::with_capacity((out_w as usize) * (out_h as usize) * 4);
    for y in 0..out_h {
        let src_y = (((y as f32) / scale) as u32).min(src_h - 1);
        for x in 0..out_w {
            let src_x = (((x as f32) / scale) as u32).min(src_w - 1);
            let pixel = frame[(src_y * src_w + src_x) as usize];
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
