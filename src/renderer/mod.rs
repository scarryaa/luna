pub mod gpu;
pub mod primatives;
pub mod surface;

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache};
use glam::{Vec2, Vec4};
use primatives::{CircleInstance, LineInstance, RectInstance};
use wgpu::util::DeviceExt;
use wgpu::{Device, Queue, TextureFormat};

use crate::layout::Rect;
pub use gpu::GpuContext;
pub use primatives::{Primative, RenderPrimative};
pub use surface::RenderSurface;

const START_CAPACITY: usize = 4 * 1024;

struct InstanceBuffer<T> {
    buf: wgpu::Buffer,
    capacity: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T: bytemuck::Pod + bytemuck::Zeroable> InstanceBuffer<T> {
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
        queue: &wgpu::Queue,
        required: usize,
        usage: wgpu::BufferUsages,
        _pool: &[T],
    ) {
        if required <= self.capacity {
            return;
        }

        let old_capacity = self.capacity;
        while self.capacity < required {
            self.capacity *= 2;
        }

        let new_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance-buf (grown)"),
            size: (self.capacity * std::mem::size_of::<T>()) as _,
            usage,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("buffer-copy-enc"),
        });
        encoder.copy_buffer_to_buffer(
            &self.buf,
            0,
            &new_buf,
            0,
            (old_capacity * std::mem::size_of::<T>()) as wgpu::BufferAddress,
        );
        queue.submit(Some(encoder.finish()));

        self.buf = new_buf;
    }

    fn upload_one(&mut self, queue: &wgpu::Queue, index: usize, val: &T) {
        let offset = (index * std::mem::size_of::<T>()) as wgpu::BufferAddress;
        queue.write_buffer(&self.buf, offset, bytemuck::bytes_of(val));
    }
}

pub type RectId = usize;
pub type LineId = usize;
pub type CircId = usize;

struct TextData {
    primative: RenderPrimative,
    glyph_rect_ids: Vec<RectId>,
    is_dirty: bool,
}

pub struct Renderer<'a> {
    gpu: GpuContext,
    surface: RenderSurface<'a>,

    screen_buf: wgpu::Buffer,
    screen_bind: wgpu::BindGroup,

    image_pipe: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_cache: HashMap<String, Arc<wgpu::BindGroup>>,
    frame_image_draws: Vec<(Rect, Arc<wgpu::BindGroup>)>,

    rect_pipe: wgpu::RenderPipeline,
    line_pipe: wgpu::RenderPipeline,
    circle_pipe: wgpu::RenderPipeline,

    rect_ibuf: InstanceBuffer<RectInstance>,
    line_ibuf: InstanceBuffer<LineInstance>,
    circle_ibuf: InstanceBuffer<CircleInstance>,

    font_system: FontSystem,
    swash_cache: SwashCache,
    text_pool: Vec<TextData>,

    rect_pool: Vec<RectInstance>,
    line_pool: Vec<LineInstance>,
    circ_pool: Vec<CircleInstance>,

    rect_dirty: Vec<(usize, RectInstance)>,
    line_dirty: Vec<(usize, LineInstance)>,
    circ_dirty: Vec<(usize, CircleInstance)>,

    frame_rect_slots: Vec<usize>,
    frame_text_ids: Vec<usize>,
    rect_call_idx: usize,
    text_call_idx: usize,

    scissor_stack: Vec<Rect>,
    scale_factor: f32,
}

impl<'a> Renderer<'a> {
    pub async fn new(window: &'a winit::window::Window, scale_factor: f32) -> Result<Self> {
        use primatives::{CircleInstance, LineInstance, RectInstance};

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

        // Layout for textures
        let texture_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });

        fn make_pipeline(
            device: &wgpu::Device,
            src: &'static str,
            label: &'static str,
            bind_layouts: &[&wgpu::BindGroupLayout],
            v_layout: wgpu::VertexBufferLayout<'static>,
            surface_fmt: wgpu::TextureFormat,
        ) -> wgpu::RenderPipeline {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(src.into()),
            });
            let pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("pipe-layout"),
                bind_group_layouts: bind_layouts,
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
            &[&screen_layout],
            RectInstance::layout(),
            surface_fmt,
        );
        let line_pipeline = make_pipeline(
            &gpu.device,
            include_str!("shaders/line.wgsl"),
            "line.wgsl",
            &[&screen_layout],
            LineInstance::layout(),
            surface_fmt,
        );
        let circle_pipeline = make_pipeline(
            &gpu.device,
            include_str!("shaders/circle.wgsl"),
            "circle.wgsl",
            &[&screen_layout],
            CircleInstance::layout(),
            surface_fmt,
        );
        let image_pipeline = make_pipeline(
            &gpu.device,
            include_str!("shaders/image.wgsl"),
            "image.wgsl",
            &[&screen_layout, &texture_bind_group_layout],
            RectInstance::layout(),
            surface_fmt,
        );

        let usage = wgpu::BufferUsages::VERTEX
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC;
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
            image_pipe: image_pipeline,
            texture_bind_group_layout,
            texture_cache: HashMap::new(),
            frame_image_draws: Vec::new(),
            rect_pipe: rect_pipeline,
            line_pipe: line_pipeline,
            circle_pipe: circle_pipeline,
            rect_ibuf,
            line_ibuf,
            circle_ibuf,
            font_system,
            swash_cache,
            text_pool: Vec::new(),
            rect_pool: Vec::new(),
            line_pool: Vec::new(),
            circ_pool: Vec::new(),
            rect_dirty: Vec::new(),
            line_dirty: Vec::new(),
            circ_dirty: Vec::new(),
            frame_rect_slots: Vec::new(),
            frame_text_ids: Vec::new(),
            rect_call_idx: 0,
            text_call_idx: 0,
            scissor_stack: Vec::new(),
            scale_factor,
        })
    }

    pub fn draw_image(&mut self, path: &str, rect: Rect) {
        if let Ok(bind_group) = self.get_texture_bind_group(path) {
            self.frame_image_draws.push((rect, bind_group.clone()));
        } else {
            log::error!("Failed to load or get texture for path: {}", path);
            self.draw_rect(rect.origin, rect.size, Vec4::new(1.0, 0.0, 1.0, 1.0));
        }
    }

    fn get_texture_bind_group(&mut self, path: &str) -> Result<Arc<wgpu::BindGroup>> {
        if let Some(bg) = self.texture_cache.get(path) {
            return Ok(bg.clone());
        }

        log::info!("Loading texture: {}", path);
        let img = image::open(path)?.to_rgba8();
        let (width, height) = img.dimensions();
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = self.gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(path),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.gpu.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &img,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = Arc::new(
            self.gpu
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                    label: Some("texture_bind_group"),
                }),
        );

        self.texture_cache
            .insert(path.to_string(), bind_group.clone());
        Ok(bind_group)
    }

    pub fn set_scale_factor(&mut self, new_factor: f32) {
        self.scale_factor = new_factor;
    }

    pub fn push_text(&mut self, p: RenderPrimative) -> usize {
        let id = self.text_pool.len();
        self.text_pool.push(TextData {
            primative: p,
            glyph_rect_ids: Vec::new(),
            is_dirty: true,
        });
        id
    }

    pub fn update_text(&mut self, id: usize, p: RenderPrimative) {
        if self.text_pool[id].primative != p {
            self.text_pool[id].primative = p;
            self.text_pool[id].is_dirty = true;
        }
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
                radius: 0.0,
                z: 0.0,
                _pad: 0.0,
            });
        });
    }

    pub fn begin_frame(&mut self) {
        self.rect_dirty.clear();
        self.line_dirty.clear();
        self.circ_dirty.clear();
        self.frame_image_draws.clear();
        self.rect_call_idx = 0;
        self.text_call_idx = 0;
    }

    pub fn end_frame(&mut self) -> Result<()> {
        let dirty_items: Vec<(usize, RenderPrimative, Vec<RectId>)> = self
            .text_pool
            .iter()
            .enumerate()
            .filter(|(_, t)| t.is_dirty)
            .map(|(i, t)| (i, t.primative.clone(), t.glyph_rect_ids.clone()))
            .collect();

        for (index, primative, old_glyph_ids) in dirty_items {
            let mut new_glyph_instances = Vec::new();
            Renderer::blit_text(
                &primative,
                &mut self.font_system,
                &mut self.swash_cache,
                &mut new_glyph_instances,
            );

            let num_new = new_glyph_instances.len();
            let num_old = old_glyph_ids.len();
            let mut new_glyph_ids = Vec::with_capacity(num_new);

            for i in 0..num_new.min(num_old) {
                let rect_id = old_glyph_ids[i];
                self.update_rect(rect_id, new_glyph_instances[i]);
                new_glyph_ids.push(rect_id);
            }

            if num_new > num_old {
                for i in num_old..num_new {
                    let rect_id = self.alloc_rect();
                    self.update_rect(rect_id, new_glyph_instances[i]);
                    new_glyph_ids.push(rect_id);
                }
            } else if num_new < num_old {
                for i in num_new..num_old {
                    let rect_id = old_glyph_ids[i];
                    self.update_rect(rect_id, RectInstance::default());
                }
            }

            let text_data = &mut self.text_pool[index];
            text_data.glyph_rect_ids = new_glyph_ids;
            text_data.is_dirty = false;
        }

        let usage = wgpu::BufferUsages::VERTEX
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC;

        self.rect_ibuf.ensure_capacity(
            &self.gpu.device,
            &self.gpu.queue,
            self.rect_pool.len(),
            usage,
            &self.rect_pool,
        );
        self.line_ibuf.ensure_capacity(
            &self.gpu.device,
            &self.gpu.queue,
            self.line_pool.len(),
            usage,
            &self.line_pool,
        );
        self.circle_ibuf.ensure_capacity(
            &self.gpu.device,
            &self.gpu.queue,
            self.circ_pool.len(),
            usage,
            &self.circ_pool,
        );

        for (idx, inst) in self.rect_dirty.drain(..) {
            self.rect_ibuf.upload_one(&self.gpu.queue, idx, &inst);
        }
        for (idx, inst) in self.line_dirty.drain(..) {
            self.line_ibuf.upload_one(&self.gpu.queue, idx, &inst);
        }
        for (idx, inst) in self.circ_dirty.drain(..) {
            self.circle_ibuf.upload_one(&self.gpu.queue, idx, &inst);
        }

        let image_instance_buffer: Option<wgpu::Buffer> = if !self.frame_image_draws.is_empty() {
            let image_instances: Vec<RectInstance> = self
                .frame_image_draws
                .iter()
                .map(|(rect, _)| RectInstance {
                    pos: rect.origin.to_array(),
                    size: rect.size.to_array(),
                    ..Default::default()
                })
                .collect();

            Some(
                self.gpu
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("temp-image-instance-buf"),
                        contents: bytemuck::cast_slice(&image_instances),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
            )
        } else {
            None
        };

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

            let size = self.surface.size();
            let full_rect = Rect::new(
                Vec2::ZERO,
                glam::vec2(size.width as f32, size.height as f32),
            );
            let logical_rect = self.scissor_stack.last().unwrap_or(&full_rect);

            let physical_x = (logical_rect.origin.x * self.scale_factor).round() as u32;
            let physical_y = (logical_rect.origin.y * self.scale_factor).round() as u32;
            let physical_w = (logical_rect.size.x * self.scale_factor).round() as u32;
            let physical_h = (logical_rect.size.y * self.scale_factor).round() as u32;

            let physical_w = physical_w.min(size.width.saturating_sub(physical_x));
            let physical_h = physical_h.min(size.height.saturating_sub(physical_y));

            rp.set_scissor_rect(physical_x, physical_y, physical_w, physical_h);

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

            if let Some(buffer) = &image_instance_buffer {
                rp.set_pipeline(&self.image_pipe);
                rp.set_vertex_buffer(0, buffer.slice(..));

                for (i, (_, bind_group)) in self.frame_image_draws.iter().enumerate() {
                    rp.set_bind_group(1, bind_group, &[]);
                    rp.draw(0..6, i as u32..(i + 1) as u32);
                }
            }
        }

        self.gpu.queue.submit(Some(enc.finish()));
        frame.present();
        Ok(())
    }

    pub fn push_scissor_rect(&mut self, rect: Rect) {
        let new_rect = if let Some(current) = self.scissor_stack.last() {
            let new_tl = current.origin.max(rect.origin);
            let new_br = (current.origin + current.size).min(rect.origin + rect.size);
            Rect::new(new_tl, (new_br - new_tl).max(Vec2::ZERO))
        } else {
            rect
        };
        self.scissor_stack.push(new_rect);
    }

    pub fn pop_scissor_rect(&mut self) {
        self.scissor_stack.pop();
    }

    pub fn draw_rounded_rect(&mut self, pos: Vec2, size: Vec2, radius: f32, colour: Vec4) {
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
                color: colour.to_array(),
                radius,
                z: 0.0,
                _pad: 0.0,
            },
        );
    }

    pub fn measure_text_segment_width(&mut self, text: &str, size: f32, char_count: usize) -> f32 {
        let metrics = Metrics::new(size, size * 1.2);
        let mut buf = Buffer::new(&mut self.font_system, metrics);
        let mut buf = buf.borrow_with(&mut self.font_system);

        let segment = text.chars().take(char_count).collect::<String>();
        buf.set_text(&segment, &Attrs::new(), Shaping::Advanced);
        buf.shape_until_scroll(true);

        buf.layout_runs().next().map_or(0.0, |run| run.line_w)
    }

    pub fn text_hit_char_index(&mut self, text: &str, size: f32, pos_x: f32) -> usize {
        let metrics = Metrics::new(size, size * 1.2);
        let mut buf = Buffer::new(&mut self.font_system, metrics);
        let mut buf = buf.borrow_with(&mut self.font_system);

        buf.set_text(text, &Attrs::new(), Shaping::Advanced);
        buf.shape_until_scroll(true);

        if let Some(cursor) = buf.hit(pos_x, size / 2.0) {
            text.char_indices()
                .take_while(|(byte_idx, _)| *byte_idx < cursor.index)
                .count()
        } else if pos_x > 0.0 {
            text.chars().count()
        } else {
            0
        }
    }

    pub fn font_system(&mut self) -> &mut FontSystem {
        &mut self.font_system
    }

    pub fn swash_cache(&mut self) -> &mut SwashCache {
        &mut self.swash_cache
    }

    pub fn font_and_swash_cache(&mut self) -> (&mut FontSystem, &mut SwashCache) {
        (&mut self.font_system, &mut self.swash_cache)
    }

    #[inline(always)]
    pub fn draw_rect(&mut self, pos: Vec2, size: Vec2, colour: Vec4) {
        self.draw_rounded_rect(pos, size, 0.0, colour);
    }

    pub fn draw_text(&mut self, text: &str, pos: Vec2, color: Vec4, size: f32) {
        if self.text_call_idx == self.frame_text_ids.len() {
            let prim = RenderPrimative::text(text, pos, color, size);
            let id = self.push_text(prim);
            self.frame_text_ids.push(id);
        }

        let id = self.frame_text_ids[self.text_call_idx];
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
