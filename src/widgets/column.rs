use glam::{Vec2, vec2};

use crate::layout::node::Node;
use crate::style::tokens::Spacing;
use crate::{Renderer, layout::Rect};

use super::{BuildCtx, Widget};

pub struct Column {
    pub children: Vec<Box<dyn Widget>>,
    pub spacing: f32,
}

impl Default for Column {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            spacing: Spacing::SM,
        }
    }
}

impl Widget for Column {
    fn build(&self, _ctx: &mut BuildCtx) -> Vec<Box<dyn Widget>> {
        self.children.iter().map(|w| w.box_clone()).collect()
    }

    fn measure(&self, max_width: f32) -> Vec2 {
        let mut w: f32 = 0.0;
        let mut h = 0.0;

        for (i, child) in self.children.iter().enumerate() {
            let sz = child.measure(max_width);
            w = w.max(sz.x);
            h += sz.y;

            if i + 1 < self.children.len() {
                h += self.spacing;
            }
        }
        vec2(w, h)
    }

    fn paint(&mut self, children: &mut [Node], layout: Rect, ren: &mut Renderer) {
        for child in children {
            if child.layout_rect.intersects(&layout) {
                child.collect(ren);
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
