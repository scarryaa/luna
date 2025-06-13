use super::GpuContext;
use wgpu::{Surface, SurfaceConfiguration, TextureFormat};

pub struct RenderSurface<'window> {
    surface: Surface<'window>,
    config: SurfaceConfiguration,
}

impl<'window> RenderSurface<'window> {
    pub fn new(gpu: &GpuContext, window: &'window winit::window::Window) -> crate::Result<Self> {
        let surface = gpu.instance.create_surface(window)?;

        let size = window.inner_size();

        let surface_caps = surface.get_capabilities(&gpu.adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&gpu.device, &config);
        Ok(Self { surface, config })
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        winit::dpi::PhysicalSize::new(self.config.width, self.config.height)
    }

    pub fn scale_factor(&self) -> f32 {
        // TODO get from window
        1.0
    }

    pub fn format(&self) -> TextureFormat {
        self.config.format
    }

    pub fn resize(&mut self, gpu: &GpuContext, new_size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&gpu.device, &self.config);
    }

    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }
}
