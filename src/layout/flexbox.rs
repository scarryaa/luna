use super::node::Node;
use crate::{
    layout::Rect,
    style::{Align, FlexDir, Justify},
};
use glam::{Vec2, vec2};

pub fn compute(
    dir: FlexDir,
    justify: Justify,
    align: Align,
    children: &mut [Node],
    avail: Vec2,
    content_origin: Vec2,
) -> Vec2 {
    let (mut main_used, mut cross_max): (f32, f32) = (0.0, 0.0);

    for n in children.iter_mut() {
        let sz = n.layout(if dir == FlexDir::Row {
            f32::INFINITY
        } else {
            avail.x
        });
        main_used += if dir == FlexDir::Row { sz.x } else { sz.y };
        cross_max = cross_max.max(if dir == FlexDir::Row { sz.y } else { sz.x });
    }

    let free = (if dir == FlexDir::Row {
        avail.x
    } else {
        avail.y
    } - main_used)
        .max(0.0);
    let total_grow: f32 = children.iter().map(|n| n.style().flex_grow).sum();
    let grow_unit = if total_grow > 0.0 {
        free / total_grow
    } else {
        0.0
    };

    let mut offset = 0.0;
    let mut gap = 0.0;
    match justify {
        Justify::Center => offset = free * 0.5,
        Justify::End => offset = free,
        Justify::SpaceBetween if children.len() > 1 => gap = free / (children.len() - 1) as f32,
        _ => {}
    }

    let mut cursor = offset;
    for n in children.iter_mut() {
        let extra = grow_unit * n.style().flex_grow;
        let size = if dir == FlexDir::Row {
            vec2(
                n.cached().x + extra,
                if align == Align::Stretch {
                    cross_max
                } else {
                    n.cached().y
                },
            )
        } else {
            vec2(cross_max, n.cached().y + extra)
        };
        let pos = content_origin
            + if dir == FlexDir::Row {
                vec2(cursor, 0.0)
            } else {
                vec2(0.0, cursor)
            };
        n.set_rect(Rect::new(pos, size));
        cursor += if dir == FlexDir::Row { size.x } else { size.y } + gap;
    }

    if dir == FlexDir::Row {
        vec2(cursor - gap, cross_max)
    } else {
        vec2(cross_max, cursor - gap)
    }
}
