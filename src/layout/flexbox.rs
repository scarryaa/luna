use super::node::Node;
use crate::style::Flex;
use crate::{
    layout::Rect,
    style::{Align, FlexDir, Justify},
};
use glam::{Vec2, vec2};

pub fn compute(flex_style: Flex, children: &mut [Node], avail: Vec2, content_origin: Vec2) -> Vec2 {
    let (mut main_used, mut cross_max): (f32, f32) = (0.0, 0.0);
    let dir = flex_style.dir;

    for n in children.iter() {
        let sz = n.cached();
        main_used += if dir == FlexDir::Row { sz.x } else { sz.y };
        cross_max = cross_max.max(if dir == FlexDir::Row { sz.y } else { sz.x });
    }

    let num_gaps = (children.len() as f32 - 1.0).max(0.0);
    if flex_style.justify != Justify::SpaceBetween {
        main_used += num_gaps * flex_style.gap;
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
    let mut gap = flex_style.gap;
    match flex_style.justify {
        Justify::Center => offset = free * 0.5,
        Justify::End => offset = free,
        Justify::SpaceBetween => {
            gap = if children.len() > 1 {
                free / num_gaps
            } else {
                0.0
            };
        }
        _ => {}
    }

    let mut cursor = offset;
    for n in children.iter_mut() {
        let extra = grow_unit * n.style().flex_grow;
        let child_size = n.cached();

        let (pos, size) = if dir == FlexDir::Row {
            let cross_avail = avail.y;
            let cross_offset = match flex_style.align {
                Align::Center => (cross_avail - child_size.y).max(0.0) / 2.0,
                Align::End => (cross_avail - child_size.y).max(0.0),
                _ => 0.0,
            };
            let p = content_origin + vec2(cursor, cross_offset);
            let s = vec2(
                child_size.x + extra,
                if flex_style.align == Align::Stretch {
                    avail.y
                } else {
                    child_size.y
                },
            );
            (p, s)
        } else {
            let cross_avail = avail.x;
            let cross_offset = match flex_style.align {
                Align::Center => (cross_avail - child_size.x).max(0.0) / 2.0,
                Align::End => (cross_avail - child_size.x).max(0.0),
                _ => 0.0,
            };
            let p = content_origin + vec2(cross_offset, cursor);
            let s = vec2(
                if flex_style.align == Align::Stretch {
                    avail.x
                } else {
                    child_size.x
                },
                child_size.y + extra,
            );
            (p, s)
        };

        n.set_rect(Rect::new(pos, size));
        cursor += (if dir == FlexDir::Row { size.x } else { size.y }) + gap;
    }

    let final_main = (cursor - gap).max(0.0);
    let final_cross = if flex_style.align == Align::Stretch {
        if dir == FlexDir::Row {
            avail.y
        } else {
            avail.x
        }
    } else {
        cross_max
    };
    if dir == FlexDir::Row {
        vec2(final_main, final_cross)
    } else {
        vec2(cross_max, cursor - gap)
    }
}
