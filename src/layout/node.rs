use glam::Vec2;
use winit::event::WindowEvent;

use crate::{
    layout::Rect,
    renderer::{
        RenderPrimative, Renderer,
        primatives::{CircleInstance, LineInstance, RectInstance},
    },
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
    prims: Vec<PrimId>,
    widget: Box<dyn Widget>,
    children: Vec<Node>,
    layout: Rect,
    dirty: bool,
}

impl Node {
    pub fn new(widget: Box<dyn Widget>, layout: Rect, ctx: &mut BuildCtx) -> Self {
        let kids = widget
            .build(ctx)
            .into_iter()
            .map(|w| Node::new(w, layout, ctx))
            .collect();

        Self {
            prims: Vec::new(),
            widget,
            children: kids,
            layout,
            dirty: true,
        }
    }

    #[inline]
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn collect(&mut self, ren: &mut Renderer) {
        if self.dirty {
            let mut fresh = Vec::<RenderPrimative>::new();
            self.widget.paint(self.layout, &mut fresh);

            while self.prims.len() < fresh.len() {
                let id = match fresh[self.prims.len()] {
                    RenderPrimative::Rectangle { .. } => PrimId::Rect(ren.alloc_rect()),
                    RenderPrimative::Line { .. } => PrimId::Line(ren.alloc_line()),
                    RenderPrimative::Circle { .. } => PrimId::Circ(ren.alloc_circle()),
                    RenderPrimative::Text { .. } => {
                        PrimId::Text(ren.push_text(fresh[self.prims.len()].clone()))
                    }
                };
                self.prims.push(id);
            }

            for (handle, prim) in self.prims.iter().zip(fresh.into_iter()) {
                match (handle, prim) {
                    (
                        PrimId::Rect(idx),
                        RenderPrimative::Rectangle {
                            position,
                            size,
                            color,
                        },
                    ) => {
                        ren.update_rect(
                            *idx,
                            RectInstance {
                                pos: position.to_array(),
                                size: size.to_array(),
                                color: color.to_array(),
                                z: 0.0,
                                _pad: 0.0,
                            },
                        );
                    }
                    (
                        PrimId::Line(idx),
                        RenderPrimative::Line {
                            start,
                            end,
                            color,
                            width,
                        },
                    ) => {
                        ren.update_line(
                            *idx,
                            LineInstance {
                                a: start.to_array(),
                                b: end.to_array(),
                                color: color.to_array(),
                                half_width: width * 0.5,
                                _pad: 0.0,
                                z: 0.0,
                            },
                        );
                    }
                    (
                        PrimId::Circ(idx),
                        RenderPrimative::Circle {
                            center,
                            radius,
                            color,
                        },
                    ) => {
                        ren.update_circle(
                            *idx,
                            CircleInstance {
                                center: center.to_array(),
                                radius,
                                _pad0: 0.0,
                                color: color.to_array(),
                                z: 0.0,
                                _pad1: 0.0,
                            },
                        );
                    }
                    (PrimId::Rect(idx), p @ RenderPrimative::Text { .. }) => {
                        ren.update_rect(*idx, (&p).into());
                    }

                    (PrimId::Text(idx), p @ RenderPrimative::Text { .. }) => {
                        ren.update_text(*idx, p);
                    }
                    _ => { /* type mismatch */ }
                }
            }

            self.dirty = false;
        }

        for child in &mut self.children {
            child.collect(ren);
        }
    }

    pub fn hit(&mut self, pt: Vec2, event: &WindowEvent) {
        for child in &mut self.children {
            child.hit(pt, event);
        }

        let inside_now = self.layout.contains(pt);

        match event {
            WindowEvent::CursorMoved { .. } => {
                if inside_now {
                    self.widget.input(event);
                    self.mark_dirty();
                } else {
                    let left = WindowEvent::CursorLeft {
                        device_id: unsafe { winit::event::DeviceId::dummy() },
                    };
                    self.widget.input(&left);
                    self.mark_dirty();
                }
            }

            WindowEvent::CursorLeft { .. } => {
                self.widget.input(event);
                self.mark_dirty();
            }

            _ => {
                if inside_now && self.widget.hit_test(pt, self.layout) {
                    self.widget.input(event);
                    self.mark_dirty();
                }
            }
        }
    }
}
