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

        self.dirty.paint_dirty = true;

        if self.dirty.self_dirty {
            self.cached_size = self.widget.measure(max_width);
        }

        if self.children.is_empty() {
            self.dirty.self_dirty = false;
            self.dirty.child_dirty = false;
            return self.cached_size;
        }

        let style = self.widget.style();
        let avail = vec2(max_width, f32::INFINITY) - style.padding_total();
        let content_origin = self.layout_rect.origin + style.padding_tl();

        match style.display {
            Display::Block => {
                let mut y = style.padding.y;
                for child in &mut self.children {
                    let sz = child.layout(avail.x);
                    let new_rect =
                        Rect::new(self.layout_rect.origin + vec2(style.padding.x, y), sz);
                    child.set_rect(new_rect);
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
                    content_origin,
                );
                self.cached_size = sz + style.padding_total();
            }
            Display::Grid => {
                let sz = crate::layout::grid::compute(
                    style.grid,
                    &mut self.children,
                    avail,
                    content_origin,
                );
                self.cached_size = sz + style.padding_total();
            }
        }

        self.dirty.self_dirty = false;
        self.dirty.child_dirty = false;
        self.cached_size
    }

    pub fn collect(&mut self, ren: &mut Renderer) {
        if self.dirty.paint_dirty {
            self.widget.paint(&mut self.children, self.layout_rect, ren);
            self.dirty.paint_dirty = false;
        } else {
            for child in &mut self.children {
                child.collect(ren);
            }
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

        let handle_event = |node: &mut Node, ctx: &mut EventCtx| {
            ctx.node_layout = node.layout_rect;
            node.widget.event(ctx, ev);

            if ctx.layout_requested {
                node.mark_dirty();
            }
            node.invalidate();
        };
        let is_target = depth == path.len();

        match phase {
            Phase::Capture => {
                handle_event(self, ctx);
                if ctx.is_stopped() || is_target {
                    return;
                }
                let idx = path[depth];
                self.children[idx].dispatch(path, depth + 1, phase, ev, ctx);
            }
            Phase::Target => {
                if is_target {
                    handle_event(self, ctx);
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
                handle_event(self, ctx);
            }
        }
    }

    pub fn route_window_event(&mut self, event: &WindowEvent, focus: &mut FocusManager) {
        match *event {
            WindowEvent::CursorMoved { position, .. } => {
                let pos = glam::vec2(position.x as f32, position.y as f32);
                self.handle_pointer_move(pos, focus);
            }

            WindowEvent::CursorLeft { .. } => {
                self.flush_pointer_leave(focus);
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if self.hover_path.is_empty() {
                    return;
                }
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
                Self::send_to_path(self, &self.hover_path.clone(), kind, focus);
            }

            WindowEvent::MouseWheel { delta, .. } => {
                if self.hover_path.is_empty() {
                    return;
                }
                let d = match delta {
                    MouseScrollDelta::LineDelta(x, y) => glam::vec2(x, y),
                    MouseScrollDelta::PixelDelta(p) => glam::vec2(p.x as f32, p.y as f32),
                };
                Self::send_to_path(
                    self,
                    &self.hover_path.clone(),
                    EventKind::Wheel { delta: d },
                    focus,
                );
            }

            WindowEvent::KeyboardInput {
                event: ref key_ev, ..
            } => {
                let focused_path = focus.path().to_vec();
                if focused_path.is_empty() {
                    return;
                }

                if let PhysicalKey::Code(key) = key_ev.physical_key {
                    let kind = match key_ev.state {
                        ElementState::Pressed => EventKind::KeyDown { key },
                        ElementState::Released => EventKind::KeyUp { key },
                    };
                    Self::send_to_path(self, &focused_path, kind, focus);
                }

                if let Some(text) = &key_ev.text {
                    if let Some(ch) = text.chars().next() {
                        Self::send_to_path(self, &focused_path, EventKind::CharInput { ch }, focus);
                    }
                }
            }

            WindowEvent::Ime(Ime::Preedit(ref s, _)) if !s.is_empty() => {
                let focused_path = focus.path().to_vec();
                if focused_path.is_empty() {
                    return;
                }
                if let Some(ch) = s.chars().next() {
                    Self::send_to_path(self, &focused_path, EventKind::CharInput { ch }, focus);
                }
            }

            _ => {}
        }
    }

    fn send_to_path(node: &mut Node, path: &[usize], kind: EventKind, focus: &mut FocusManager) {
        for &phase in &[Phase::Capture, Phase::Target, Phase::Bubble] {
            let mut ctx = EventCtx::new(phase, focus, path, Rect::new(Vec2::ZERO, Vec2::ZERO));
            node.dispatch(path, 0, phase, &kind, &mut ctx);
        }
    }

    fn handle_pointer_move(&mut self, pos: Vec2, focus: &mut FocusManager) {
        let mut new_path = Vec::<usize>::new();
        if !self.hittest(pos, &mut new_path) {
            self.flush_pointer_leave(focus);
            return;
        }

        if self.hover_path == new_path {
            Self::send_to_path(self, &new_path, EventKind::PointerMove { pos }, focus);
            return;
        }

        let old_path_clone = self.hover_path.clone();
        if !old_path_clone.is_empty() {
            Self::send_to_path(self, &old_path_clone, EventKind::PointerLeave, focus);
        }

        Self::send_to_path(self, &new_path, EventKind::PointerMove { pos }, focus);

        self.hover_path = new_path;
    }

    fn flush_pointer_leave(&mut self, focus: &mut FocusManager) {
        if self.hover_path.is_empty() {
            return;
        }

        let old_path_clone = self.hover_path.clone();
        Self::send_to_path(self, &old_path_clone, EventKind::PointerLeave, focus);

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
        if self.layout_rect == r {
            return;
        }
        self.layout_rect = r;
        self.invalidate();
        self.mark_child_dirty();
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

    pub fn mark_dirty(&mut self) {
        self.dirty.self_dirty = true;
        self.dirty.paint_dirty = true;
    }
}
