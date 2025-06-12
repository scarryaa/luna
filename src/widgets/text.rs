use glam::Vec4;

use crate::{layout::Rect, renderer::RenderPrimative};

use super::base::Widget;

#[derive(Clone)]
pub struct Text {
    pub content: String,
    pub color: Vec4,
    pub size: f32,
}

impl Widget for Text {
    fn paint(&self, layout: Rect, out: &mut Vec<RenderPrimative>) {
        out.push(RenderPrimative::text(
            &self.content,
            layout.origin,
            self.color,
            self.size,
        ));
    }
}
