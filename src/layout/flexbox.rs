use super::node::Node;
use crate::layout::Rect;
use crate::style::{Align, Flex, FlexDir, Justify};
use glam::{Vec2, vec2};

pub fn compute(flex_style: Flex, children: &mut [Node], avail: Vec2, content_origin: Vec2) -> Vec2 {
    let dir = flex_style.dir;
    let is_row = dir == FlexDir::Row;

    let mut total_child_main_size = 0.0;
    let mut total_grow_factor = 0.0;
    let mut cross_max: f32 = 0.0;

    for child in children.iter() {
        let child_size = child.cached();
        let main_size = if is_row { child_size.x } else { child_size.y };
        let cross_size = if is_row { child_size.y } else { child_size.x };

        total_child_main_size += main_size;
        total_grow_factor += child.style().flex_grow;
        cross_max = cross_max.max(cross_size);
    }

    let num_gaps = (children.len() as f32 - 1.0).max(0.0);
    let gap_size = flex_style.gap * num_gaps;
    total_child_main_size += gap_size;

    let main_avail = if is_row { avail.x } else { avail.y };
    let free_space = (main_avail - total_child_main_size).max(0.0);
    let grow_unit = if total_grow_factor > 0.0 {
        free_space / total_grow_factor
    } else {
        0.0
    };

    let mut main_cursor = match flex_style.justify {
        Justify::Center => free_space / 2.0,
        Justify::End => free_space,
        Justify::SpaceBetween => 0.0,
        _ => 0.0,
    };

    let space_between_gap = if flex_style.justify == Justify::SpaceBetween && children.len() > 1 {
        free_space / num_gaps
    } else {
        flex_style.gap
    };

    for child in children.iter_mut() {
        let child_cached_size = child.cached();
        let grow = child.style().flex_grow * grow_unit;

        let final_size = if is_row {
            vec2(
                child_cached_size.x + grow,
                if flex_style.align == Align::Stretch {
                    avail.y
                } else {
                    child_cached_size.y
                },
            )
        } else {
            vec2(
                if flex_style.align == Align::Stretch {
                    avail.x
                } else {
                    child_cached_size.x
                },
                child_cached_size.y + grow,
            )
        };

        let cross_avail = if is_row { avail.y } else { avail.x };
        let cross_size = if is_row { final_size.y } else { final_size.x };
        let cross_offset = match flex_style.align {
            Align::Center => (cross_avail - cross_size) / 2.0,
            Align::End => cross_avail - cross_size,
            _ => 0.0,
        };

        let pos = if is_row {
            content_origin + vec2(main_cursor, cross_offset)
        } else {
            content_origin + vec2(cross_offset, main_cursor)
        };

        child.set_rect(Rect::new(pos, final_size));

        let main_size = if is_row { final_size.x } else { final_size.y };
        main_cursor += main_size + space_between_gap;
    }

    let final_main_size = if children.is_empty() {
        0.0
    } else {
        main_cursor - space_between_gap
    };

    let final_cross_size = if flex_style.align == Align::Stretch {
        if is_row { avail.y } else { avail.x }
    } else {
        cross_max
    };

    if is_row {
        vec2(final_main_size, final_cross_size)
    } else {
        vec2(final_cross_size, final_main_size)
    }
}
