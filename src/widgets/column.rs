use glam::{Vec2, vec2};

use crate::layout::node::Node;
use crate::{Renderer, style::Theme};

use super::{BuildCtx, Widget};

pub struct Column {
    pub children: Vec<Box<dyn Widget>>,
    pub spacing: f32,
}

impl Default for Column {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            spacing: 4.0,
        }
    }
}

impl Widget for Column {
    fn build(&self, _ctx: &mut BuildCtx) -> Vec<Box<dyn Widget>> {
        self.children.iter().map(|w| w.box_clone()).collect()
    }

    fn measure(
        &self,
        max_width: f32,
        theme: &Theme,
        font_system: &mut cosmic_text::FontSystem,
    ) -> Vec2 {
        let mut w: f32 = 0.0;
        let mut h = 0.0;

        for (i, child) in self.children.iter().enumerate() {
            let sz = child.measure(max_width, theme, font_system);
            w = w.max(sz.x);
            h += sz.y;

            if i + 1 < self.children.len() {
                h += self.spacing;
            }
        }
        vec2(w, h)
    }

    fn paint(&mut self, node: &mut Node, ren: &mut Renderer, theme: &Theme) {
        for child in &mut node.children {
            if child.layout_rect.intersects(&node.layout_rect) {
                child.collect(ren, theme);
            }
        }
    }
}

impl Clone for Column {
    fn clone(&self) -> Self {
        Column {
            children: self.children.iter().map(|w| w.box_clone()).collect(),
            spacing: self.spacing,
        }
    }
}
