use crate::{layout::node::Node, style::Theme};
use glam::{Vec2, Vec4, vec2};

use crate::{
    renderer::{RectId, Renderer, primatives::RectInstance},
    style::{Align, Display, FlexDir, Justify, Style},
    widgets::{BuildCtx, Widget},
};

#[derive(Clone, Default)]
pub struct Element {
    pub style: Style,
    pub children: Vec<Box<dyn Widget>>,
    bg_id: Option<RectId>,
}

impl Element {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fill_cross(mut self, v: bool) -> Self {
        self.style.flex.fill_cross = v;
        self
    }

    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.children.push(Box::new(widget));
        self
    }

    pub fn children(mut self, widgets: impl IntoIterator<Item = Box<dyn Widget>>) -> Self {
        self.children.extend(widgets);
        self
    }

    pub fn display(mut self, display: Display) -> Self {
        self.style.display = display;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.style.width = Some(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.style.height = Some(height);
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.style.padding = Vec2::splat(padding);
        self
    }

    pub fn background_color(mut self, color: impl Into<Vec4>) -> Self {
        self.style.background_color = Some(color.into());
        self
    }

    pub fn flex_direction(mut self, dir: FlexDir) -> Self {
        self.style.flex.dir = dir;
        self
    }

    pub fn justify_content(mut self, justify: Justify) -> Self {
        self.style.flex.justify = justify;
        self
    }

    pub fn align_items(mut self, align: Align) -> Self {
        self.style.flex.align = align;
        self
    }

    pub fn grid_cols(mut self, cols: u16) -> Self {
        self.style.grid.cols = cols;
        self
    }

    pub fn grid_row_height(mut self, height: f32) -> Self {
        self.style.grid.row_height = height;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.style.flex.gap = gap;
        self.style.grid.gap = vec2(gap, gap);
        self
    }
}

impl Widget for Element {
    fn build(&self, _ctx: &mut BuildCtx) -> Vec<Box<dyn Widget>> {
        self.children.clone()
    }

    fn style(&self) -> Style {
        self.style
    }

    fn measure(
        &self,
        _max_width: f32,
        _theme: &Theme,
        _font_system: &mut cosmic_text::FontSystem,
    ) -> Vec2 {
        Vec2::ZERO
    }

    fn paint(&mut self, node: &mut Node, ren: &mut Renderer, theme: &Theme) {
        if let Some(color) = self.style.background_color {
            let id = *self.bg_id.get_or_insert_with(|| ren.alloc_rect());
            ren.update_rect(
                id,
                RectInstance {
                    pos: node.layout_rect.origin.to_array(),
                    size: node.layout_rect.size.to_array(),
                    color: color.to_array(),
                    ..Default::default()
                },
            );
        }

        for child in &mut node.children {
            if child.layout_rect.intersects(&node.layout_rect) {
                child.collect(ren, theme);
            }
        }
    }
}
