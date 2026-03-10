use anyhow::{Result, anyhow};
use slint::Image;
use slint::wgpu_28::wgpu;

const FRAME_WIDTH: u32 = 256;
const FRAME_HEIGHT: u32 = 240;
const FRAME_BYTES_PER_PIXEL: u32 = 4;

pub struct GameRenderer {
    queue: wgpu::Queue,
    texture: wgpu::Texture,
    width: u32,
    height: u32,
}

impl GameRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<(Self, Image)> {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nesium_slint_game_frame"),
            size: wgpu::Extent3d {
                width: FRAME_WIDTH,
                height: FRAME_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let renderer = Self {
            queue: queue.clone(),
            texture: texture.clone(),
            width: FRAME_WIDTH,
            height: FRAME_HEIGHT,
        };

        renderer.clear_to_black();

        let image = Image::try_from(texture).map_err(|err| anyhow!("{err}"))?;
        Ok((renderer, image))
    }

    pub fn upload_rgba_frame(
        &mut self,
        rgba: &[u8],
        width: usize,
        height: usize,
        pitch_bytes: usize,
    ) -> bool {
        if width as u32 != self.width || height as u32 != self.height {
            return false;
        }

        let expected_row_bytes = (self.width * FRAME_BYTES_PER_PIXEL) as usize;
        if pitch_bytes < expected_row_bytes || rgba.len() < pitch_bytes.saturating_mul(height) {
            return false;
        }

        self.queue.write_texture(
            self.texture.as_image_copy(),
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(pitch_bytes as u32),
                rows_per_image: Some(self.height),
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        true
    }

    fn clear_to_black(&self) {
        let blank = vec![0u8; (self.width * self.height * FRAME_BYTES_PER_PIXEL) as usize];
        self.queue.write_texture(
            self.texture.as_image_copy(),
            &blank,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.width * FRAME_BYTES_PER_PIXEL),
                rows_per_image: Some(self.height),
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
    }
}
