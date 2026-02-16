use getset::{Getters, MutGetters};
use ironboyadvance_core::{VIEWPORT_HEIGHT, VIEWPORT_WIDTH};
use sdl2::{
    Sdl,
    image::{self, InitFlag},
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
    render::{Canvas, TextureCreator},
    ttf::{self, Sdl2TtfContext},
    video::{Window, WindowContext},
};
use thiserror::Error;

const SCALE: u32 = 6;
const FPS_FONT_SIZE: u16 = 24;
const FPS_PADDING: i32 = 10;
const FONT_PATH: &str = "media/gbboot-alpm.ttf";

#[derive(Error, Debug)]
pub enum WindowError {
    #[error("Failed to create video subsystem: {0}")]
    VideoSubsystemError(String),
    #[error("Failed to initialize image context: {0}")]
    ImageInitError(String),
    #[error("Failed to initialize TTF context: {0}")]
    TtfInitError(String),
    #[error("Failed to create window: {0}")]
    WindowBuildError(#[from] sdl2::video::WindowBuildError),
    #[error("Failed to create canvas from window: {0}")]
    CanvasBuildError(#[from] sdl2::IntegerOrSdlError),
    #[error("There was a canvas error: {0}")]
    CanvasError(String),
    #[error("There was a texture error: {0}")]
    TextureError(String),
    #[error("Failed to load font: {0}")]
    FontLoadError(String),
    #[error("Failed to render text: {0}")]
    TextRenderError(String),
}

#[derive(Getters, MutGetters)]
pub struct WindowManager {
    #[getset(get = "pub", get_mut = "pub")]
    main_canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    ttf_context: Sdl2TtfContext,
}

impl WindowManager {
    pub fn new(sdl_context: &Sdl) -> Result<WindowManager, WindowError> {
        image::init(InitFlag::PNG).map_err(WindowError::ImageInitError)?;
        let ttf_context = ttf::init().map_err(WindowError::TtfInitError)?;

        let video_subsystem = sdl_context.video().map_err(WindowError::VideoSubsystemError)?;
        let window = video_subsystem
            .window("Iron Boy", (VIEWPORT_WIDTH as u32) * SCALE, (VIEWPORT_HEIGHT as u32) * SCALE)
            .position_centered()
            .resizable()
            .opengl()
            .build()?;

        let main_canvas = window.into_canvas().accelerated().build()?;
        let texture_creator = main_canvas.texture_creator();

        Ok(Self {
            main_canvas,
            texture_creator,
            ttf_context,
        })
    }

    pub fn render_screen(&mut self, data: &[u32], fps: Option<f64>) -> Result<(), WindowError> {
        self.main_canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.main_canvas.clear();

        {
            let mut texture = self
                .texture_creator
                .create_texture_streaming(PixelFormatEnum::RGB24, VIEWPORT_WIDTH as u32, VIEWPORT_HEIGHT as u32)
                .map_err(|e| WindowError::TextureError(e.to_string()))?;

            texture
                .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                    for y in 0..VIEWPORT_HEIGHT as usize {
                        for x in 0..VIEWPORT_WIDTH as usize {
                            let i = y * VIEWPORT_WIDTH as usize + x;
                            let offset = y * pitch + x * 3;
                            let pixel = data[i];
                            buffer[offset] = ((pixel >> 16) & 0xFF) as u8; // R
                            buffer[offset + 1] = ((pixel >> 8) & 0xFF) as u8; // G
                            buffer[offset + 2] = (pixel & 0xFF) as u8; // B
                        }
                    }
                })
                .map_err(WindowError::CanvasError)?;

            let (window_width, window_height) = self.main_canvas.output_size().map_err(WindowError::CanvasError)?;
            let scale_x = window_width as f32 / VIEWPORT_WIDTH as f32;
            let scale_y = window_height as f32 / VIEWPORT_HEIGHT as f32;
            let scale = scale_x.min(scale_y);

            let rendered_width = (VIEWPORT_WIDTH as f32 * scale) as u32;
            let rendered_height = (VIEWPORT_HEIGHT as f32 * scale) as u32;
            let offset_x = (window_width - rendered_width) / 2;
            let offset_y = (window_height - rendered_height) / 2;

            let dst_rect = Rect::new(offset_x as i32, offset_y as i32, rendered_width, rendered_height);
            self.main_canvas
                .copy(&texture, None, dst_rect)
                .map_err(WindowError::CanvasError)?;
        }

        if let Some(fps_value) = fps {
            self.render_fps_overlay(fps_value)?;
        }

        self.main_canvas.present();
        Ok(())
    }

    fn render_fps_overlay(&mut self, fps: f64) -> Result<(), WindowError> {
        let font = self
            .ttf_context
            .load_font(FONT_PATH, FPS_FONT_SIZE)
            .map_err(WindowError::FontLoadError)?;

        let fps_text = format!("{:.1} FPS", fps);
        let surface = font
            .render(&fps_text)
            .blended(Color::RGB(0, 255, 0))
            .map_err(|e| WindowError::TextRenderError(e.to_string()))?;

        let texture = self
            .texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| WindowError::TextureError(e.to_string()))?;

        let (window_width, window_height) = self.main_canvas.output_size().map_err(WindowError::CanvasError)?;
        let text_width = surface.width();
        let text_height = surface.height();

        let x = (window_width - text_width) as i32 - FPS_PADDING;
        let y = (window_height - text_height) as i32 - FPS_PADDING;

        let dst_rect = Rect::new(x, y, text_width, text_height);
        self.main_canvas
            .copy(&texture, None, dst_rect)
            .map_err(WindowError::CanvasError)?;

        Ok(())
    }
}
