use super::BuildCtx;
use crate::style::Style;
use crate::windowing::events::{EventCtx, EventKind};
use crate::{Renderer, layout::Rect};
use glam::Vec2;
use winit::event::WindowEvent;

pub trait Widget: WidgetClone {
    fn build(&self, _ctx: &mut BuildCtx) -> Vec<Box<dyn Widget>> {
        Vec::new()
    }

    fn measure(&self, max_width: f32) -> Vec2;

    fn paint(&self, layout: Rect, ren: &mut Renderer);

    fn event(&mut self, _ctx: &mut EventCtx, _ev: &EventKind) {}

    fn hit_test(&self, _pt: Vec2, _layout: Rect) -> bool {
        false
    }

    fn input(&mut self, _event: &WindowEvent) {}

    fn style(&self) -> Style {
        Style::default()
    }
}

pub trait WidgetClone {
    fn box_clone(&self) -> Box<dyn Widget>;
}

impl<T: Widget + Clone + 'static> WidgetClone for T {
    fn box_clone(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Widget> {
    fn clone(&self) -> Self {
        self.as_ref().box_clone()
    }
}
