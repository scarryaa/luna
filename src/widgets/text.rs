use glam::{Vec2, Vec4, vec2};

use crate::{Renderer, layout::Rect};

use super::base::Widget;

#[derive(Clone)]
pub struct Text {
    pub content: String,
    pub color: Vec4,
    pub size: f32,
}

impl Widget for Text {
    fn measure(&self, _max_width: f32) -> Vec2 {
        // very rough: average glyph width ≈ 0.6 × font size
        vec2(self.content.len() as f32 * self.size * 0.6, self.size)
    }

    fn paint(&self, layout: Rect, ren: &mut Renderer) {
        ren.draw_text(&self.content, layout.origin, self.color, self.size);
    }
}
