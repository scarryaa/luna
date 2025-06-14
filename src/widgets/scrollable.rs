use glam::{Vec2, vec2};

use crate::{
    Widget,
    layout::{Rect, node::Node},
    renderer::Renderer,
    windowing::events::{EventCtx, EventKind},
};

#[derive(Clone)]
pub struct Scrollable {
    child: Box<dyn Widget>,
    offset: Vec2,
    child_size: Vec2,
}

impl Scrollable {
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            offset: Vec2::ZERO,
            child_size: Vec2::ZERO,
        }
    }
}

impl Widget for Scrollable {
    fn build(&self, _ctx: &mut crate::widgets::BuildCtx) -> Vec<Box<dyn Widget>> {
        vec![self.child.clone()]
    }

    fn measure(&self, max_width: f32) -> Vec2 {
        vec2(max_width, f32::INFINITY)
    }

    fn event(&mut self, ctx: &mut EventCtx, ev: &EventKind) {
        if let EventKind::Wheel { delta } = ev {
            let scroll_amount = delta.y * 20.0;
            self.offset.y -= scroll_amount;

            self.offset.y = self.offset.y.max(0.0);
            let max_offset_y = (self.child_size.y - ctx.node_layout.size.y).max(0.0);
            self.offset.y = self.offset.y.min(max_offset_y);

            ctx.request_layout();
        }
    }

    fn paint(&mut self, node: &mut Node, ren: &mut Renderer) {
        let layout = node.layout_rect;
        ren.push_scissor_rect(layout);

        if let Some(child) = node.children.first_mut() {
            self.child_size = child.cached();

            let child_pos = layout.origin - self.offset;
            child.set_rect(Rect::new(child_pos, self.child_size));

            child.layout(layout.size.x);
            child.collect(ren);
        }

        ren.pop_scissor_rect();
    }
}
