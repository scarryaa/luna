use glam::{Vec2, vec2};
use winit::event::{DeviceId, WindowEvent};

use crate::{
    layout::{Dirty, Rect},
    renderer::Renderer,
    widgets::{BuildCtx, Widget},
};

#[derive(Copy, Clone)]
pub enum PrimId {
    Rect(usize),
    Line(usize),
    Circ(usize),
    Text(usize),
}

pub struct Node {
    widget: Box<dyn Widget>,
    children: Vec<Node>,

    layout_rect: Rect, // absolute rect in parent space
    cached_size: Vec2, // result of last `measure`
    dirty: Dirty,
}

impl Node {
    pub fn new(widget: Box<dyn Widget>, layout: Rect, ctx: &mut BuildCtx) -> Self {
        let kids = widget
            .build(ctx)
            .into_iter()
            .map(|w| Node::new(w, layout, ctx))
            .collect();

        Self {
            widget,
            children: kids,

            layout_rect: layout,
            cached_size: layout.size,
            dirty: Dirty {
                self_dirty: true,
                child_dirty: true,
            },
        }
    }

    pub fn invalidate(&mut self) {
        if !self.dirty.self_dirty {
            self.dirty.self_dirty = true;
        }
    }

    pub fn mark_child_dirty(&mut self) {
        self.dirty.child_dirty = true;
    }

    pub fn layout(&mut self, max_width: f32) -> Vec2 {
        if !self.dirty.self_dirty && !self.dirty.child_dirty {
            return self.cached_size;
        }

        if self.dirty.self_dirty {
            self.cached_size = self.widget.measure(max_width);
        }

        if self.dirty.self_dirty || self.dirty.child_dirty {
            let mut y = 0.0;
            for child in &mut self.children {
                let sz = child.layout(self.cached_size.x);
                child.layout_rect = Rect::new(self.layout_rect.origin + vec2(0.0, y), sz);
                y += sz.y;
            }
        }

        self.dirty.child_dirty = false;
        self.cached_size
    }

    pub fn collect(&mut self, ren: &mut Renderer) {
        if self.dirty.self_dirty || self.dirty.child_dirty {
            self.widget.paint(self.layout_rect, ren);
            self.dirty.self_dirty = false;
        }

        for child in &mut self.children {
            child.collect(ren);
        }
    }

    pub fn hit(&mut self, pt: Vec2, event: &WindowEvent) {
        for child in &mut self.children {
            child.hit(pt, event);
        }

        let inside = self.layout_rect.contains(pt);

        match event {
            WindowEvent::CursorMoved { .. } => {
                if inside {
                    self.widget.input(event);
                    self.invalidate();
                } else {
                    let left = WindowEvent::CursorLeft {
                        device_id: unsafe { DeviceId::dummy() },
                    };
                    self.widget.input(&left);
                    self.invalidate();
                }
            }
            WindowEvent::CursorLeft { .. } => {
                self.widget.input(event);
                self.invalidate();
            }
            _ => {
                if inside && self.widget.hit_test(pt, self.layout_rect) {
                    self.widget.input(event);
                    self.invalidate();
                }
            }
        }
    }
}
