use super::node::Node;
use crate::layout::Rect;
use crate::style::{Align, Flex, FlexDir, Justify};
use glam::{Vec2, vec2};

pub fn compute(flex_style: Flex, children: &mut [Node], avail: Vec2, content_origin: Vec2) -> Vec2 {
    let dir = flex_style.dir;
    let is_row = dir == FlexDir::Row;

    let mut content_main_size = 0.0;
    let mut total_grow_factor = 0.0;
    let mut max_cross_size: f32 = 0.0;

    for child in children.iter() {
        let child_size = child.cached();
        let main_size = if is_row { child_size.x } else { child_size.y };
        let cross_size = if is_row { child_size.y } else { child_size.x };

        content_main_size += main_size;
        total_grow_factor += child.style().flex_grow;
        max_cross_size = max_cross_size.max(cross_size);
    }

    let num_gaps = (children.len() as f32 - 1.0).max(0.0);
    content_main_size += num_gaps * flex_style.gap;

    let main_avail = if is_row { avail.x } else { avail.y };
    let free_space = (main_avail - content_main_size).max(0.0);

    let final_container_main_size = if flex_style.justify == Justify::Start {
        content_main_size
    } else {
        main_avail
    };

    let cross_avail = if is_row { avail.y } else { avail.x };
    let final_container_cross_size = if flex_style.fill_cross || flex_style.align != Align::Start {
        cross_avail
    } else {
        max_cross_size
    };

    let has_grow = total_grow_factor > 0.0;

    let grow_unit = if has_grow {
        free_space / total_grow_factor
    } else {
        0.0
    };

    let mut main_cursor = if has_grow {
        0.0
    } else {
        match flex_style.justify {
            Justify::Center => free_space / 2.0,
            Justify::End => free_space,
            _ => 0.0,
        }
    };

    let space_between_gap =
        if !has_grow && flex_style.justify == Justify::SpaceBetween && children.len() > 1 {
            free_space / num_gaps
        } else {
            flex_style.gap
        };

    for child in children.iter_mut() {
        let child_cached_size = child.cached();
        let grow = child.style().flex_grow * grow_unit;

        let child_main_size = (if is_row {
            child_cached_size.x
        } else {
            child_cached_size.y
        }) + grow;
        let mut child_cross_size = if is_row {
            child_cached_size.y
        } else {
            child_cached_size.x
        };

        if flex_style.align == Align::Stretch {
            child_cross_size = final_container_cross_size;
        }

        let cross_offset = match flex_style.align {
            Align::Center => (final_container_cross_size - child_cross_size) / 2.0,
            Align::End => final_container_cross_size - child_cross_size,
            _ => 0.0,
        };

        let pos = if is_row {
            content_origin + vec2(main_cursor, cross_offset)
        } else {
            content_origin + vec2(cross_offset, main_cursor)
        };

        let final_size = if is_row {
            vec2(child_main_size, child_cross_size)
        } else {
            vec2(child_cross_size, child_main_size)
        };
        child.set_rect(Rect::new(pos, final_size));

        main_cursor += child_main_size + space_between_gap;
    }

    if is_row {
        vec2(final_container_main_size, final_container_cross_size)
    } else {
        vec2(final_container_cross_size, final_container_main_size)
    }
}
