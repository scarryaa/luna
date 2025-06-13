use glam::{Vec2, vec2};
use winit::event::{ElementState, MouseScrollDelta};
use winit::{
    event::{Ime, WindowEvent},
    keyboard::PhysicalKey,
};

use crate::{
    dbg_ev,
    layout::{Dirty, Rect},
    renderer::Renderer,
    style::Display,
    widgets::{BuildCtx, Widget},
    windowing::events::{EventCtx, EventKind, FocusManager, Phase},
};

#[derive(Copy, Clone)]
pub enum PrimId {
    Rect(usize),
    Line(usize),
    Circ(usize),
    Text(usize),
}

pub struct Node {
    pub widget: Box<dyn Widget>,
    children: Vec<Node>,

    pub layout_rect: Rect,
    pub cached_size: Vec2,
    dirty: Dirty,
    hover_path: Vec<usize>,
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
                paint_dirty: true,
            },
            hover_path: Vec::new(),
        }
    }

    pub fn layout(&mut self, max_width: f32) -> Vec2 {
        if !self.dirty.self_dirty && !self.dirty.child_dirty {
            return self.cached_size;
        }

        if self.dirty.self_dirty {
            self.cached_size = self.widget.measure(max_width);
        }

        if self.children.is_empty() {
            self.dirty.child_dirty = false;
            return self.cached_size;
        }

        let style = self.widget.style();
        let avail = vec2(max_width, f32::INFINITY) - style.padding_total();

        match style.display {
            Display::Block => {
                let mut y = style.padding.y;
                for child in &mut self.children {
                    let sz = child.layout(avail.x);
                    child.layout_rect =
                        Rect::new(self.layout_rect.origin + vec2(style.padding.x, y), sz);
                    y += sz.y;
                }
                self.cached_size = vec2(max_width, y) + style.padding_total();
            }
            Display::Flex => {
                let sz = crate::layout::flexbox::compute(
                    style.flex.dir,
                    style.flex.justify,
                    style.flex.align,
                    &mut self.children,
                    avail,
                );
                self.cached_size = sz + style.padding_total();
            }
            Display::Grid => {
                let sz = crate::layout::grid::compute(style.grid, &mut self.children, avail);
                self.cached_size = sz + style.padding_total();
            }
        }

        self.dirty.child_dirty = false;
        self.cached_size
    }

    pub fn collect(&mut self, ren: &mut Renderer) {
        if self.dirty.paint_dirty {
            self.widget.paint(self.layout_rect, ren);
            self.dirty.paint_dirty = false;
        }

        for child in &mut self.children {
            child.collect(ren);
        }
    }

    fn hittest(&self, pt: Vec2, path: &mut Vec<usize>) -> bool {
        if !self.layout_rect.contains(pt) {
            return false;
        }

        for (i, child) in self.children.iter().enumerate() {
            if child.hittest(pt, path) {
                path.insert(0, i);
                return true;
            }
        }
        true
    }

    fn dispatch(
        &mut self,
        path: &[usize],
        depth: usize,
        phase: Phase,
        ev: &EventKind,
        ctx: &mut EventCtx,
    ) {
        log::trace!(
            "dispatch  phase={:?} depth={} idx={} targ={} ev={}",
            phase,
            depth,
            if depth < path.len() { path[depth] } else { 999 },
            depth == path.len(),
            dbg_ev!(ev),
        );

        let is_target = depth == path.len();

        match phase {
            Phase::Capture => {
                self.widget.event(ctx, ev);
                self.invalidate();
                if ctx.is_stopped() || is_target {
                    return;
                }
                let idx = path[depth];
                self.children[idx].dispatch(path, depth + 1, phase, ev, ctx);
            }

            Phase::Target => {
                if is_target {
                    self.widget.event(ctx, ev);
                    self.invalidate();
                } else {
                    let idx = path[depth];
                    self.children[idx].dispatch(path, depth + 1, phase, ev, ctx);
                }
            }

            Phase::Bubble => {
                if !is_target {
                    let idx = path[depth];
                    self.children[idx].dispatch(path, depth + 1, phase, ev, ctx);
                    if ctx.is_stopped() {
                        return;
                    }
                }
                self.widget.event(ctx, ev);
                self.invalidate();
            }
        }
    }

    pub fn route_window_event(&mut self, event: &WindowEvent, focus: &mut FocusManager) {
        match *event {
            WindowEvent::CursorMoved { position, .. } => {
                let pos = glam::vec2(position.x as f32, position.y as f32);
                self.handle_pointer_move(pos, focus);
            }

            WindowEvent::CursorLeft { .. } => self.flush_pointer_leave(focus),

            WindowEvent::MouseInput { state, button, .. } => {
                if self.hover_path.is_empty() {
                    return;
                }
                let tgt_path = self.hover_path.clone();
                let kind = match state {
                    ElementState::Pressed => EventKind::PointerDown {
                        button,
                        pos: Vec2::ZERO,
                    },
                    ElementState::Released => EventKind::PointerUp {
                        button,
                        pos: Vec2::ZERO,
                    },
                };
                Self::send_to_path(self, &tgt_path, kind, focus);
            }

            WindowEvent::MouseWheel { delta, .. } => {
                if self.hover_path.is_empty() {
                    return;
                }
                let tgt_path = self.hover_path.clone();
                let d = match delta {
                    MouseScrollDelta::LineDelta(x, y) => glam::vec2(x, y),
                    MouseScrollDelta::PixelDelta(p) => glam::vec2(p.x as f32, p.y as f32),
                };
                Self::send_to_path(self, &tgt_path, EventKind::Wheel { delta: d }, focus);
            }

            WindowEvent::KeyboardInput {
                event: ref key_ev, ..
            } => {
                let key = match key_ev.physical_key {
                    PhysicalKey::Code(k) => k,
                    _ => return,
                };
                let kind = match key_ev.state {
                    ElementState::Pressed => EventKind::KeyDown { key },
                    ElementState::Released => EventKind::KeyUp { key },
                };

                let path = focus.path().to_vec();
                if !path.is_empty() {
                    Self::send_to_path(self, &path, kind, focus);

                    if let Some(text) = &key_ev.text {
                        if let Some(ch) = text.chars().next() {
                            Self::send_to_path(self, &path, EventKind::CharInput { ch }, focus);
                        }
                    }
                }
            }

            WindowEvent::Ime(Ime::Preedit(ref s, _)) if !s.is_empty() => {
                let path = focus.path().to_vec();
                if !path.is_empty() {
                    let ch = s.chars().next().unwrap();
                    Self::send_to_path(self, &path, EventKind::CharInput { ch }, focus);
                }
            }

            _ => {}
        }
    }

    fn send_to_path(node: &mut Node, path: &[usize], kind: EventKind, focus: &mut FocusManager) {
        for &phase in &[Phase::Capture, Phase::Target, Phase::Bubble] {
            let mut ctx = EventCtx::new(phase, focus, path);
            node.dispatch(path, 0, phase, &kind, &mut ctx);
        }
    }

    fn handle_pointer_move(&mut self, pos: Vec2, focus: &mut FocusManager) {
        let mut new_path = Vec::<usize>::new();
        if !self.hittest(pos, &mut new_path) {
            self.flush_pointer_leave(focus);
            return;
        }

        let leave_path = self.hover_path.clone();
        for depth in (0..leave_path.len()).rev() {
            let slice = &leave_path[..depth + 1];
            let mut ctx = EventCtx::new(Phase::Target, focus, slice);
            self.dispatch(
                slice,
                depth,
                Phase::Target,
                &EventKind::PointerLeave,
                &mut ctx,
            );
        }

        let move_ev = EventKind::PointerMove { pos };
        for &phase in &[Phase::Capture, Phase::Target, Phase::Bubble] {
            let mut ctx = EventCtx::new(phase, focus, &new_path);
            self.dispatch(&new_path, 0, phase, &move_ev, &mut ctx);
        }

        self.hover_path = new_path;
    }

    fn flush_pointer_leave(&mut self, focus: &mut FocusManager) {
        let leave_path = self.hover_path.clone();
        for depth in (0..leave_path.len()).rev() {
            let slice = &leave_path[..depth + 1];
            let mut ctx = EventCtx::new(Phase::Target, focus, slice);
            self.dispatch(
                slice,
                depth,
                Phase::Target,
                &EventKind::PointerLeave,
                &mut ctx,
            );
        }
        self.hover_path.clear();
    }

    fn invalidate(&mut self) {
        if !self.dirty.paint_dirty {
            self.dirty.paint_dirty = true;
        }
    }

    pub fn style(&self) -> crate::Style {
        self.widget.style()
    }

    pub fn set_rect(&mut self, r: Rect) {
        self.layout_rect = r;
    }

    pub fn origin(&self) -> Vec2 {
        self.layout_rect.origin
    }

    pub fn cached(&self) -> Vec2 {
        self.cached_size
    }

    pub fn mark_child_dirty(&mut self) {
        self.dirty.child_dirty = true;
    }
}
