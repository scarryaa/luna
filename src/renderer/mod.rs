pub mod gpu;
pub mod primatives;
pub mod surface;

use cosmic_text::{Attrs, Color, FontSystem, Metrics, Shaping, SwashCache};
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
            label: Some("instance-buf"),
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
        // grow ×2 until big enough
        while self.capacity < required {
            self.capacity *= 2;
        }
        self.buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance-buf (grown)"),
            size: (self.capacity * std::mem::size_of::<T>()) as _,
            usage,
            mapped_at_creation: false,
        });
    }

    fn upload_one(&mut self, queue: &wgpu::Queue, index: usize, val: &T) {
        let offset = (index * std::mem::size_of::<T>()) as wgpu::BufferAddress;
        queue.write_buffer(&self.buf, offset, bytemuck::bytes_of(val));
    }
}

pub type RectId = usize;
pub type LineId = usize;
pub type CircId = usize;

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

    font_system: FontSystem,
    swash_cache: SwashCache,
    text_pool: Vec<RenderPrimative>,
    text_slots: Vec<usize>,

    rect_pool: Vec<RectInstance>,
    line_pool: Vec<LineInstance>,
    circ_pool: Vec<CircleInstance>,

    rect_dirty: Vec<(usize, RectInstance)>,
    line_dirty: Vec<(usize, LineInstance)>,
    circ_dirty: Vec<(usize, CircleInstance)>,

    frame_rect_slots: Vec<usize>,
    frame_text_slots: Vec<usize>,
    rect_call_idx: usize,
    text_call_idx: usize,
}

impl<'a> Renderer<'a> {
    pub async fn new(window: &'a winit::window::Window) -> crate::Result<Self> {
        use primatives::{CircleInstance, LineInstance, RectInstance};
        use wgpu::util::DeviceExt;

        let gpu = GpuContext::new().await?;
        let surf = RenderSurface::new(&gpu, window)?;
        let size = window.inner_size();
        let surface_fmt = surf.format();

        let screen_buf = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("screen-uniform"),
                contents: bytemuck::cast_slice(&[size.width as f32, size.height as f32]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let screen_layout = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("screen-layout"),
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
            label: Some("screen-bind"),
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
                label: Some("pipe-layout"),
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

        let usage = wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST;
        let rect_ibuf = InstanceBuffer::<RectInstance>::new(&gpu.device, usage);
        let line_ibuf = InstanceBuffer::<LineInstance>::new(&gpu.device, usage);
        let circle_ibuf = InstanceBuffer::<CircleInstance>::new(&gpu.device, usage);

        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        Ok(Self {
            gpu,
            surface: surf,
            screen_buf,
            screen_bind,
            rect_pipe: rect_pipeline,
            line_pipe: line_pipeline,
            circle_pipe: circle_pipeline,

            rect_ibuf,
            line_ibuf,
            circle_ibuf,

            font_system,
            swash_cache,
            text_slots: Vec::new(),

            rect_pool: Vec::new(),
            line_pool: Vec::new(),
            circ_pool: Vec::new(),
            text_pool: Vec::new(),

            rect_dirty: Vec::new(),
            line_dirty: Vec::new(),
            circ_dirty: Vec::new(),

            frame_rect_slots: Vec::new(),
            frame_text_slots: Vec::new(),
            rect_call_idx: 0,
            text_call_idx: 0,
        })
    }

    pub fn push_text(&mut self, p: RenderPrimative) -> usize {
        let id = self.text_pool.len();
        self.text_pool.push(p);
        id
    }

    pub fn update_text(&mut self, id: usize, p: RenderPrimative) {
        self.text_pool[id] = p;
    }

    pub fn alloc_rect(&mut self) -> RectId {
        let id = self.rect_pool.len();
        self.rect_pool.push(RectInstance::default());
        self.rect_dirty.push((id, self.rect_pool[id]));
        id
    }

    pub fn update_rect(&mut self, id: RectId, data: RectInstance) {
        if self.rect_pool[id] != data {
            self.rect_pool[id] = data;
            self.rect_dirty.push((id, data));
        }
    }

    pub fn alloc_line(&mut self) -> LineId {
        let id = self.line_pool.len();
        self.line_pool.push(LineInstance::default());
        self.line_dirty.push((id, self.line_pool[id]));
        id
    }

    pub fn update_line(&mut self, id: LineId, data: LineInstance) {
        if self.line_pool[id] != data {
            self.line_pool[id] = data;
            self.line_dirty.push((id, data));
        }
    }

    pub fn alloc_circle(&mut self) -> CircId {
        let id = self.circ_pool.len();
        self.circ_pool.push(CircleInstance::default());
        self.circ_dirty.push((id, self.circ_pool[id]));
        id
    }

    pub fn update_circle(&mut self, id: CircId, data: CircleInstance) {
        if self.circ_pool[id] != data {
            self.circ_pool[id] = data;
            self.circ_dirty.push((id, data));
        }
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

    pub fn begin_frame(&mut self) {
        self.rect_dirty.clear();
        self.line_dirty.clear();
        self.circ_dirty.clear();

        self.rect_call_idx = 0;
        self.text_call_idx = 0;
    }

    pub fn end_frame(&mut self) -> crate::Result<()> {
        let mut glyph_rects = Vec::<RectInstance>::new();
        {
            let font = &mut self.font_system;
            let swash = &mut self.swash_cache;

            for prim in &self.text_pool {
                Renderer::blit_text(prim, font, swash, &mut glyph_rects);
            }
        }

        while self.text_slots.len() < glyph_rects.len() {
            let id = self.rect_pool.len();
            self.rect_pool.push(RectInstance::default());
            self.text_slots.push(id);
        }

        self.text_slots.truncate(glyph_rects.len());

        for (glyph_idx, inst) in glyph_rects.into_iter().enumerate() {
            let id = self.text_slots[glyph_idx];
            if self.rect_pool[id] != inst {
                self.rect_pool[id] = inst;
                self.rect_dirty.push((id, inst));
            }
        }

        let usage = wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST;
        self.rect_ibuf
            .ensure_capacity(&self.gpu.device, self.rect_pool.len(), usage);
        self.line_ibuf
            .ensure_capacity(&self.gpu.device, self.line_pool.len(), usage);
        self.circle_ibuf
            .ensure_capacity(&self.gpu.device, self.circ_pool.len(), usage);

        for (idx, inst) in self.rect_dirty.drain(..) {
            self.rect_ibuf.upload_one(&self.gpu.queue, idx, &inst);
        }
        for (idx, inst) in self.line_dirty.drain(..) {
            self.line_ibuf.upload_one(&self.gpu.queue, idx, &inst);
        }
        for (idx, inst) in self.circ_dirty.drain(..) {
            self.circle_ibuf.upload_one(&self.gpu.queue, idx, &inst);
        }

        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut enc = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("main-enc"),
            });

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
            rp.draw(0..6, 0..self.rect_pool.len() as _);

            rp.set_pipeline(&self.line_pipe);
            rp.set_vertex_buffer(0, self.line_ibuf.buf.slice(..));
            rp.draw(0..6, 0..self.line_pool.len() as _);

            rp.set_pipeline(&self.circle_pipe);
            rp.set_vertex_buffer(0, self.circle_ibuf.buf.slice(..));
            rp.draw(0..6, 0..self.circ_pool.len() as _);
        }

        self.gpu.queue.submit(Some(enc.finish()));
        frame.present();
        Ok(())
    }

    pub fn draw_rect(&mut self, pos: Vec2, size: Vec2, color: Vec4) {
        if self.rect_call_idx == self.frame_rect_slots.len() {
            let id = self.alloc_rect();
            self.frame_rect_slots.push(id);
        }
        let id = self.frame_rect_slots[self.rect_call_idx];
        self.rect_call_idx += 1;

        self.update_rect(
            id,
            RectInstance {
                pos: pos.to_array(),
                size: size.to_array(),
                color: color.to_array(),
                z: 0.0,
                _pad: 0.0,
            },
        );
    }

    pub fn draw_text(&mut self, text: &str, pos: Vec2, color: Vec4, size: f32) {
        if self.text_call_idx == self.frame_text_slots.len() {
            let prim = RenderPrimative::text(text, pos, color, size);
            let id = self.push_text(prim.clone());
            self.text_slots.push(id);
            self.frame_text_slots.push(id);
        }
        let id = self.frame_text_slots[self.text_call_idx];
        self.text_call_idx += 1;

        let prim = RenderPrimative::text(text, pos, color, size);
        self.update_text(id, prim);
    }

    pub fn device(&self) -> &Device {
        &self.gpu.device
    }
    pub fn queue(&self) -> &Queue {
        &self.gpu.queue
    }
}
