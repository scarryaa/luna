use glam::{Vec2, Vec4, vec2};

use crate::{
    Renderer,
    layout::Rect,
    style::tokens::{Colour, Typography},
};

use super::base::Widget;

#[derive(Clone)]
pub struct Text {
    pub content: String,
    pub color: Vec4,
    pub size: f32,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            content: String::new(),
            color: Vec4::from(Colour::TEXT),
            size: Typography::BODY,
        }
    }
}

impl Widget for Text {
    fn measure(&self, _max_width: f32) -> Vec2 {
        vec2(self.content.len() as f32 * self.size * 0.6, self.size)
    }

    fn paint(&mut self, layout: Rect, ren: &mut Renderer) {
        ren.draw_text(&self.content, layout.origin, self.color, self.size);
    }
}
