use std::{cell::RefCell, rc::Rc};

use glam::Vec2;

use crate::{layout::node::Node, renderer::Renderer, widgets::Widget};

#[derive(Clone)]
pub struct Canvas {
    on_paint: Rc<RefCell<Box<dyn FnMut(&mut Renderer)>>>,
}

impl Canvas {
    pub fn new(on_paint: impl FnMut(&mut Renderer) + 'static) -> Self {
        Self {
            on_paint: Rc::new(RefCell::new(Box::new(on_paint))),
        }
    }
}

impl Widget for Canvas {
    fn build(&self, _ctx: &mut crate::widgets::BuildCtx) -> Vec<Box<dyn Widget>> {
        Vec::new()
    }

    fn measure(&self, _max_width: f32) -> Vec2 {
        Vec2::ZERO
    }

    fn paint(&mut self, _node: &mut Node, ren: &mut Renderer) {
        (self.on_paint.borrow_mut())(ren);
    }
}
