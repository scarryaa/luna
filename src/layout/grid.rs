use super::node::Node;
use crate::{
    layout::Rect,
    style::{Grid, Theme},
};
use cosmic_text::FontSystem;
use glam::{Vec2, vec2};

pub fn compute(
    grid: Grid,
    children: &mut [Node],
    avail: Vec2,
    content_origin: Vec2,
    theme: &Theme,
    font_system: &mut FontSystem,
) -> Vec2 {
    let cw = (avail.x - grid.gap.x * (grid.cols - 1) as f32) / grid.cols as f32;
    let mut max_y: f32 = 0.0;

    for (i, n) in children.iter_mut().enumerate() {
        let c = i as u16 % grid.cols;
        let r = i as u16 / grid.cols;
        let offset = vec2(
            cw * c as f32 + grid.gap.x * c as f32,
            r as f32 * (grid.row_height + grid.gap.y),
        );
        let pos = content_origin + offset;
        let sz = n.layout(cw, theme, font_system);
        n.set_rect(Rect::new(pos, vec2(cw, grid.row_height.max(sz.y))));
        max_y = max_y.max(pos.y + grid.row_height.max(sz.y) - content_origin.y);
    }

    vec2(avail.x, max_y)
}
