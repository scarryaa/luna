use glam::{Vec2, Vec4, vec2};

use crate::{
    layout::{Rect, node::Node},
    renderer::Renderer,
    signals::ReadSignal,
    style::tokens::{Colour, Typography},
};

use super::base::Widget;

#[derive(Clone)]
pub struct Text {
    pub content: ReadSignal<String>,
    pub color: Vec4,
    pub size: f32,
}

impl Text {
    pub fn new(content: impl Into<String>) -> Self {
        let (read, _) = crate::signals::create_signal(content.into());
        Self {
            content: read,
            color: Vec4::from(Colour::TEXT),
            size: Typography::BODY,
        }
    }
}

impl Widget for Text {
    fn measure(&self, _max_width: f32) -> Vec2 {
        let content_str = self.content.get();
        vec2(content_str.len() as f32 * self.size * 0.6, self.size)
    }

    fn paint(&mut self, _children: &mut [Node], layout: Rect, ren: &mut Renderer) {
        ren.draw_text(&self.content.get(), layout.origin, self.color, self.size);
    }
}
