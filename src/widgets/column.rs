use crate::{layout::Rect, renderer::RenderPrimative};

use super::{BuildCtx, Widget};

pub struct Column {
    pub children: Vec<Box<dyn Widget>>,
    pub spacing: f32,
}

impl Widget for Column {
    fn build(&self, _ctx: &mut BuildCtx) -> Vec<Box<dyn Widget>> {
        self.children.iter().map(|w| w.box_clone()).collect()
    }

    fn paint(&self, _layout: Rect, _out: &mut Vec<RenderPrimative>) {}
}

impl Clone for Column {
    fn clone(&self) -> Self {
        Column {
            children: self.children.iter().map(|w| w.box_clone()).collect(),
            spacing: self.spacing,
        }
    }
}
