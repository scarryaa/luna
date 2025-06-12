pub mod gpu;
pub mod primatives;
pub mod surface;

use cosmic_text::{Attrs, Color, Metrics, Shaping};
use cosmic_text::{FontSystem, SwashCache};
use glam::{Vec2, Vec4};
use primatives::{CircleInstance, LineInstance, RectInstance};
use wgpu::{Device, Queue, TextureFormat};

pub use gpu::GpuContext;
pub use primatives::{Primative, RenderPrimative};
pub use surface::RenderSurface;

pub struct Renderer<'a> {
    gpu: GpuContext,
    surface: RenderSurface<'a>,
    screen_buf: wgpu::Buffer,
    screen_bind: wgpu::BindGroup,
    rect_pipeline: wgpu::RenderPipeline,
    line_pipeline: wgpu::RenderPipeline,
    circle_pipeline: wgpu::RenderPipeline,

    rect_instances: Vec<RectInstance>,
    line_instances: Vec<LineInstance>,
    circle_instances: Vec<CircleInstance>,

    font_system: FontSystem,
    swash_cache: SwashCache,
    text_prims: Vec<RenderPrimative>,
}

impl<'a> Renderer<'a> {
    pub async fn new(window: &'a winit::window::Window) -> crate::Result<Self> {
        use wgpu::util::DeviceExt;
        let gpu = GpuContext::new().await?;
        let surf = RenderSurface::new(&gpu, window)?;
        let size = window.inner_size();

        let surface_fmt = surf.format();

        let screen_buf = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("screen uniform"),
                contents: bytemuck::cast_slice(&[size.width as f32, size.height as f32]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let screen_layout = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("screen layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let screen_bind = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("screen bind"),
            layout: &screen_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_buf.as_entire_binding(),
            }],
        });

        fn make_pipeline(
            device: &wgpu::Device,
            src: &'static str,
            label: &'static str,
            bind_layout: &wgpu::BindGroupLayout,
            v_layout: wgpu::VertexBufferLayout<'static>,
            surface_fmt: wgpu::TextureFormat,
        ) -> wgpu::RenderPipeline {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(src.into()),
            });

            // create pipeline layout
            let pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("pipeline-layout"),
                bind_group_layouts: &[bind_layout],
                push_constant_ranges: &[],
            });

            // assemble final pipeline
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(src),
                layout: Some(&pipe_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: "vs_main",
                    buffers: &[v_layout],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_fmt,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            })
        }

        let rect_pipeline = make_pipeline(
            &gpu.device,
            include_str!("shaders/rect.wgsl"),
            "rect.wgsl",
            &screen_layout,
            RectInstance::layout(),
            surface_fmt,
        );

        let line_pipeline = make_pipeline(
            &gpu.device,
            include_str!("shaders/line.wgsl"),
            "line.wgsl",
            &screen_layout,
            LineInstance::layout(),
            surface_fmt,
        );

        let circle_pipeline = make_pipeline(
            &gpu.device,
            include_str!("shaders/circle.wgsl"),
            "circle.wgsl",
            &screen_layout,
            CircleInstance::layout(),
            surface_fmt,
        );

        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        Ok(Self {
            gpu,
            surface: surf,
            screen_buf,
            screen_bind,
            rect_pipeline,
            line_pipeline,
            circle_pipeline,
            rect_instances: Vec::new(),
            line_instances: Vec::new(),
            circle_instances: Vec::new(),
            font_system: font_system,
            swash_cache: swash_cache,
            text_prims: Vec::new(),
        })
    }

    pub fn surface_format(&self) -> TextureFormat {
        self.surface.format()
    }

    pub fn resize(&mut self, new: winit::dpi::PhysicalSize<u32>) {
        self.surface.resize(&self.gpu, new);
        let data = [new.width as f32, new.height as f32];
        self.gpu
            .queue
            .write_buffer(&self.screen_buf, 0, bytemuck::cast_slice(&data));
    }

    pub fn begin_frame(&mut self) -> crate::Result<()> {
        self.rect_instances.clear();
        self.line_instances.clear();
        self.circle_instances.clear();
        Ok(())
    }

    pub fn draw_primative(&mut self, prim: RenderPrimative) {
        match &prim {
            RenderPrimative::Rectangle { .. } => self.rect_instances.push((&prim).into()),
            RenderPrimative::Line { .. } => self.line_instances.push((&prim).into()),
            RenderPrimative::Circle { .. } => self.circle_instances.push((&prim).into()),
            RenderPrimative::Text { .. } => self.text_prims.push(prim),
        }
    }

    pub fn draw_rect(&mut self, position: Vec2, size: Vec2, color: Vec4) {
        let primative = RenderPrimative::Rectangle {
            position,
            size,
            color,
        };

        self.draw_primative(primative);
    }

    pub fn draw_text(&mut self, text: &str, position: Vec2, color: Vec4, size: f32) {
        let primative = RenderPrimative::Text {
            text: text.to_string(),
            position,
            color,
            size,
        };

        self.draw_primative(primative);
    }

    fn blit_text(
        prim: &RenderPrimative,
        font_system: &mut FontSystem,
        swash: &mut SwashCache,
        out: &mut Vec<RectInstance>,
    ) {
        let RenderPrimative::Text {
            text,
            position,
            color,
            size,
        } = prim
        else {
            return;
        };

        let metrics = Metrics::new(*size, size * 1.2);
        let mut buf = cosmic_text::Buffer::new(font_system, metrics);
        let mut buf = buf.borrow_with(font_system);

        buf.set_text(text, &Attrs::new(), Shaping::Advanced);
        buf.shape_until_scroll(true);

        let fg = Color::rgba(
            (color.x * 255.0) as u8,
            (color.y * 255.0) as u8,
            (color.z * 255.0) as u8,
            (color.w * 255.0) as u8,
        );

        buf.draw(swash, fg, |x, y, w, h, rgba| {
            let pos = *position + Vec2::new(x as f32, y as f32);
            out.push(RectInstance {
                pos: pos.to_array(),
                size: [w as f32, h as f32],
                color: [
                    rgba.r() as f32 / 255.0,
                    rgba.g() as f32 / 255.0,
                    rgba.b() as f32 / 255.0,
                    rgba.a() as f32 / 255.0,
                ],
            });
        });
    }

    pub fn end_frame(&mut self) -> crate::Result<()> {
        use wgpu::util::DeviceExt;

        let mut extra_rects = Vec::new();
        for prim in &self.text_prims {
            Self::blit_text(
                prim,
                &mut self.font_system,
                &mut self.swash_cache,
                &mut extra_rects,
            );
        }
        self.rect_instances.extend(extra_rects);

        let rect_buf = self
            .gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("rect inst"),
                contents: bytemuck::cast_slice(&self.rect_instances),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let line_buf = self
            .gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("line inst"),
                contents: bytemuck::cast_slice(&self.line_instances),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let circle_buf = self
            .gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("circle inst"),
                contents: bytemuck::cast_slice(&self.circle_instances),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });
        {
            let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            rp.set_bind_group(0, &self.screen_bind, &[]);

            rp.set_pipeline(&self.rect_pipeline);
            rp.set_vertex_buffer(0, rect_buf.slice(..));
            rp.draw(0..6, 0..self.rect_instances.len() as _);

            rp.set_pipeline(&self.line_pipeline);
            rp.set_vertex_buffer(0, line_buf.slice(..));
            rp.draw(0..6, 0..self.line_instances.len() as _);

            rp.set_pipeline(&self.circle_pipeline);
            rp.set_vertex_buffer(0, circle_buf.slice(..));
            rp.draw(0..6, 0..self.circle_instances.len() as _);
        }
        self.gpu.queue.submit(std::iter::once(enc.finish()));
        output.present();
        Ok(())
    }

    pub fn device(&self) -> &Device {
        &self.gpu.device
    }

    pub fn queue(&self) -> &Queue {
        &self.gpu.queue
    }
}
