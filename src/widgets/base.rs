use super::BuildCtx;
use crate::{layout::Rect, renderer::RenderPrimative};
use glam::Vec2;
use winit::event::WindowEvent;

pub trait Widget: WidgetClone {
    fn build(&self, _ctx: &mut BuildCtx) -> Vec<Box<dyn Widget>> {
        Vec::new()
    }

    fn paint(&self, layout: Rect, out: &mut Vec<RenderPrimative>);

    fn hit_test(&self, _pt: Vec2, _layout: Rect) -> bool {
        false
    }

    fn input(&mut self, _event: &WindowEvent) {}
}

pub trait WidgetClone {
    fn box_clone(&self) -> Box<dyn Widget>;
}

impl<T> WidgetClone for T
where
    T: Widget + Clone + 'static,
{
    fn box_clone(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}
