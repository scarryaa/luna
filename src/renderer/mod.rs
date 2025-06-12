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

const START_CAPACITY: usize = 4 * 1024;

struct InstanceBuffer<T> {
    buf: wgpu::Buffer,
    capacity: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T: bytemuck::Pod> InstanceBuffer<T> {
    fn new(device: &wgpu::Device, usage: wgpu::BufferUsages) -> Self {
        let buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance buf"),
            size: (START_CAPACITY * std::mem::size_of::<T>()) as _,
            usage,
            mapped_at_creation: false,
        });
        Self {
            buf,
            capacity: START_CAPACITY,
            _marker: Default::default(),
        }
    }

    fn ensure_capacity(
        &mut self,
        device: &wgpu::Device,
        required: usize,
        usage: wgpu::BufferUsages,
    ) {
        if required <= self.capacity {
            return;
        }
        // grow 2× until big enough
        while self.capacity < required {
            self.capacity *= 2;
        }
        self.buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance buf (grown)"),
            size: (self.capacity * std::mem::size_of::<T>()) as _,
            usage,
            mapped_at_creation: false,
        });
    }

    fn upload(&mut self, queue: &wgpu::Queue, data: &[T]) {
        queue.write_buffer(&self.buf, 0, bytemuck::cast_slice(data));
    }
}

pub struct Renderer<'a> {
    gpu: GpuContext,
    surface: RenderSurface<'a>,

    screen_buf: wgpu::Buffer,
    screen_bind: wgpu::BindGroup,

    rect_pipe: wgpu::RenderPipeline,
    line_pipe: wgpu::RenderPipeline,
    circle_pipe: wgpu::RenderPipeline,

    rect_ibuf: InstanceBuffer<RectInstance>,
    line_ibuf: InstanceBuffer<LineInstance>,
    circle_ibuf: InstanceBuffer<CircleInstance>,

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

            let pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("pipeline-layout"),
                bind_group_layouts: &[bind_layout],
                push_constant_ranges: &[],
            });

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
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

        let usage = wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST;
        let rect_ibuf = InstanceBuffer::<RectInstance>::new(&gpu.device, usage);
        let line_ibuf = InstanceBuffer::<LineInstance>::new(&gpu.device, usage);
        let circle_ibuf = InstanceBuffer::<CircleInstance>::new(&gpu.device, usage);

        Ok(Self {
            gpu,
            surface: surf,
            screen_buf,
            screen_bind,
            rect_pipe: rect_pipeline,
            line_pipe: line_pipeline,
            circle_pipe: circle_pipeline,
            rect_instances: Vec::new(),
            line_instances: Vec::new(),
            circle_instances: Vec::new(),
            rect_ibuf,
            line_ibuf,
            circle_ibuf,
            font_system,
            swash_cache,
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

    pub fn begin_frame(&mut self) {
        self.rect_instances.clear();
        self.line_instances.clear();
        self.circle_instances.clear();
        self.text_prims.clear();
    }

    pub fn draw_primative(&mut self, prim: RenderPrimative) {
        match &prim {
            RenderPrimative::Rectangle { .. } => self.rect_instances.push((&prim).into()),
            RenderPrimative::Line { .. } => self.line_instances.push((&prim).into()),
            RenderPrimative::Circle { .. } => self.circle_instances.push((&prim).into()),
            RenderPrimative::Text { .. } => self.text_prims.push(prim),
        }
    }

    pub fn draw_rect(&mut self, p: Vec2, s: Vec2, c: Vec4) {
        self.draw_primative(RenderPrimative::rectangle(p, s, c))
    }
    pub fn draw_text(&mut self, t: &str, p: Vec2, c: Vec4, s: f32) {
        self.draw_primative(RenderPrimative::text(t, p, c, s))
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
                z: 0.0,
                _pad: 0.0,
            });
        });
    }

    pub fn end_frame(&mut self) -> crate::Result<()> {
        /* ----- shape text into extra rects ----------------------------------- */
        let mut extra = Vec::new();
        for p in &self.text_prims {
            Self::blit_text(p, &mut self.font_system, &mut self.swash_cache, &mut extra);
        }
        self.rect_instances.extend(extra);

        /* ----- sort by z if you start assigning layers ----------------------- */
        self.rect_instances.sort_by(|a, b| a.z.total_cmp(&b.z));
        self.line_instances.sort_by(|a, b| a.z.total_cmp(&b.z));
        self.circle_instances.sort_by(|a, b| a.z.total_cmp(&b.z));

        /* ----- grow / upload persistent buffers ------------------------------ */
        let usage = wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST;
        self.rect_ibuf
            .ensure_capacity(&self.gpu.device, self.rect_instances.len(), usage);
        self.line_ibuf
            .ensure_capacity(&self.gpu.device, self.line_instances.len(), usage);
        self.circle_ibuf
            .ensure_capacity(&self.gpu.device, self.circle_instances.len(), usage);

        self.rect_ibuf.upload(&self.gpu.queue, &self.rect_instances);
        self.line_ibuf.upload(&self.gpu.queue, &self.line_instances);
        self.circle_ibuf
            .upload(&self.gpu.queue, &self.circle_instances);

        /* ----- begin render pass --------------------------------------------- */
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut enc = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("enc") });

        {
            let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main-pass"),
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

            rp.set_pipeline(&self.rect_pipe);
            rp.set_vertex_buffer(0, self.rect_ibuf.buf.slice(..));
            rp.draw(0..6, 0..self.rect_instances.len() as _);

            rp.set_pipeline(&self.line_pipe);
            rp.set_vertex_buffer(0, self.line_ibuf.buf.slice(..));
            rp.draw(0..6, 0..self.line_instances.len() as _);

            rp.set_pipeline(&self.circle_pipe);
            rp.set_vertex_buffer(0, self.circle_ibuf.buf.slice(..));
            rp.draw(0..6, 0..self.circle_instances.len() as _);
        }

        self.gpu.queue.submit(Some(enc.finish()));
        output.present();
        Ok(())
    }

    pub fn device(&self) -> &Device {
        &self.gpu.device
    }

    pub fn queue(&self) -> &Queue {
        &self.gpu.queue
    }

    pub fn with_scissor<F: FnOnce(&mut wgpu::RenderPass)>(
        rp: &mut wgpu::RenderPass,
        rect: Option<(u32, u32, u32, u32)>,
        target: winit::dpi::PhysicalSize<u32>,
        f: F,
    ) {
        if let Some((x, y, w, h)) = rect {
            rp.set_scissor_rect(x, y, w, h);
            f(rp);
            rp.set_scissor_rect(0, 0, target.width, target.height);
        } else {
            f(rp)
        }
    }
}
