use glam::{Vec2, Vec4};

#[derive(Debug, Clone, PartialEq)]
pub enum RenderPrimative {
    Rectangle {
        position: Vec2,
        size: Vec2,
        color: Vec4,
    },
    Text {
        text: String,
        position: Vec2,
        color: Vec4,
        size: f32,
    },
    Line {
        start: Vec2,
        end: Vec2,
        color: Vec4,
        width: f32,
    },
    Circle {
        center: Vec2,
        radius: f32,
        color: Vec4,
    },
}

#[repr(C)]
#[derive(Copy, Clone, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectInstance {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub radius: f32,
    pub z: f32,
    pub _pad: f32,
}

impl RectInstance {
    const ATTRS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x2,   // pos
        1 => Float32x2,   // size
        2 => Float32x4,   // color
        3 => Float32,     // radius
        4 => Float32      // z
    ];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as _,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineInstance {
    pub a: [f32; 2],
    pub b: [f32; 2],
    pub color: [f32; 4],
    pub half_width: f32,
    pub _pad: f32,
    pub z: f32,
}

impl LineInstance {
    const ATTRS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x2,   // a
        1 => Float32x2,   // b
        2 => Float32x4,   // color
        3 => Float32,     // half_width
        4 => Float32      // z
    ];
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as _,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleInstance {
    pub center: [f32; 2],
    pub radius: f32,
    pub _pad0: f32,
    pub color: [f32; 4],
    pub z: f32,
    pub _pad1: f32,
}

impl CircleInstance {
    const ATTRS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x2,   // center
        1 => Float32,     // radius
        2 => Float32,     // pad
        3 => Float32x4,   // color
        4 => Float32      // z
    ];
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as _,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRS,
        }
    }
}

impl From<&RenderPrimative> for RectInstance {
    fn from(p: &RenderPrimative) -> Self {
        match p {
            RenderPrimative::Rectangle {
                position,
                size,
                color,
            } => Self {
                pos: position.to_array(),
                size: size.to_array(),
                color: color.to_array(),
                z: 0.0,
                _pad: 0.0,
                radius: 0.0,
            },
            _ => unreachable!(),
        }
    }
}

impl From<&RenderPrimative> for LineInstance {
    fn from(p: &RenderPrimative) -> Self {
        match p {
            RenderPrimative::Line {
                start,
                end,
                color,
                width,
            } => Self {
                a: start.to_array(),
                b: end.to_array(),
                color: color.to_array(),
                half_width: *width * 0.5,
                _pad: 0.0,
                z: 0.0,
            },
            _ => unreachable!(),
        }
    }
}

impl From<&RenderPrimative> for CircleInstance {
    fn from(p: &RenderPrimative) -> Self {
        match p {
            RenderPrimative::Circle {
                center,
                radius,
                color,
            } => Self {
                center: center.to_array(),
                radius: *radius,
                _pad0: 0.0,
                color: color.to_array(),
                z: 0.0,
                _pad1: 0.0,
            },
            _ => unreachable!(),
        }
    }
}

// Trait for objects that can be converted to render primatives
pub trait Primative {
    // Convert this object to render primatives
    fn to_primatives(&self) -> Vec<RenderPrimative>;
}

impl Primative for RenderPrimative {
    fn to_primatives(&self) -> Vec<RenderPrimative> {
        vec![self.clone()]
    }
}

impl RenderPrimative {
    // Create a new rectangle primative
    pub fn rectangle(position: Vec2, size: Vec2, color: Vec4) -> Self {
        Self::Rectangle {
            position,
            size,
            color,
        }
    }

    // Create a new text primative
    pub fn text(text: impl Into<String>, position: Vec2, color: Vec4, size: f32) -> Self {
        Self::Text {
            text: text.into(),
            position,
            color,
            size,
        }
    }

    // Create a new line primative
    pub fn line(start: Vec2, end: Vec2, color: Vec4, width: f32) -> Self {
        Self::Line {
            start,
            end,
            color,
            width,
        }
    }

    // Create a new circle primative
    pub fn circle(center: Vec2, radius: f32, color: Vec4) -> Self {
        Self::Circle {
            center,
            radius,
            color,
        }
    }

    // Get the bounding box of this primative
    pub fn bounding_box(&self) -> (Vec2, Vec2) {
        match self {
            Self::Rectangle { position, size, .. } => (*position, *position + *size),
            Self::Text { position, size, .. } => {
                // TODO measure w/font metrics
                let text_size = Vec2::new(size * 0.6, *size);
                (*position, *position + text_size)
            }
            Self::Line {
                start, end, width, ..
            } => {
                let min = start.min(*end) - Vec2::splat(*width * 0.5);
                let max = start.max(*end) + Vec2::splat(*width * 0.5);
                (min, max)
            }
            Self::Circle { center, radius, .. } => {
                let offset = Vec2::splat(*radius);
                (*center - offset, *center + offset)
            }
        }
    }
}
