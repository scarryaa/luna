use std::mem;

use cosmic_text::FontSystem;
use glam::{Vec2, vec2};
use winit::event::{ElementState, Ime, MouseScrollDelta, WindowEvent};

use crate::signals::{NodeId, ScopedNodeContext};
use crate::style::Theme;
use crate::{
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
    pub id: NodeId,
    pub widget: Box<dyn Widget>,
    pub children: Vec<Node>,

    pub layout_rect: Rect,
    pub cached_size: Vec2,
    dirty: Dirty,
    hover_path: Vec<usize>,
}

impl Node {
    pub fn new(widget: Box<dyn Widget>, layout: Rect, ctx: &mut BuildCtx) -> Self {
        let id = NodeId::new();
        let kids = widget
            .build(ctx)
            .into_iter()
            .map(|w| Node::new(w, layout, ctx))
            .collect();

        Self {
            id,
            widget,
            children: kids,
            layout_rect: layout,
            cached_size: Vec2::ZERO,
            dirty: Dirty {
                self_dirty: true,
                child_dirty: true,
                paint_dirty: true,
            },
            hover_path: Vec::new(),
        }
    }

    pub fn layout(&mut self, max_width: f32, theme: &Theme, font_system: &mut FontSystem) -> Vec2 {
        if !self.dirty.self_dirty && !self.dirty.child_dirty {
            return self.cached_size;
        }

        self.dirty.paint_dirty = true;
        let style = self.widget.style();
        let padding_size = style.padding_total();

        let content_size: Vec2;

        if !self.children.is_empty() {
            let child_max_width = if let Some(w) = style.width {
                w - padding_size.x
            } else {
                max_width - padding_size.x
            };
            for child in &mut self.children {
                child.layout(child_max_width, theme, font_system);
            }

            let avail = vec2(max_width, self.layout_rect.size.y) - padding_size;
            let content_origin = self.layout_rect.origin + style.padding_tl();

            content_size = match style.display {
                Display::Flex => crate::layout::flexbox::compute(
                    style.flex,
                    &mut self.children,
                    avail,
                    content_origin,
                ),
                Display::Grid => crate::layout::grid::compute(
                    style.grid,
                    &mut self.children,
                    avail,
                    content_origin,
                    theme,
                    font_system,
                ),
                Display::Block => {
                    let mut y = 0.0;
                    let mut max_x: f32 = 0.0;
                    for child in &mut self.children {
                        let sz = child.cached();
                        let new_rect = Rect::new(content_origin + vec2(0.0, y), sz);
                        child.set_rect(new_rect);
                        y += sz.y;
                        max_x = max_x.max(sz.x);
                    }
                    vec2(max_x, y)
                }
            };
        } else {
            content_size = self
                .widget
                .measure(max_width - padding_size.x, theme, font_system);
        }

        let mut final_size = content_size + padding_size;

        if let Some(w) = style.width {
            final_size.x = w;
        }
        if let Some(h) = style.height {
            final_size.y = h;
        }

        self.cached_size = final_size;

        self.dirty.self_dirty = false;
        self.dirty.child_dirty = false;
        self.cached_size
    }

    pub fn collect(&mut self, ren: &mut Renderer, theme: &Theme) {
        let _guard = ScopedNodeContext::new(self.id);

        let mut widget = mem::replace(
            &mut self.widget,
            Box::new(crate::widgets::Element::default()),
        );

        widget.paint(self, ren, theme);

        self.widget = widget;
        self.dirty.paint_dirty = false;
    }

    pub fn mark_dirty_by_id(&mut self, target_id: NodeId) -> bool {
        if self.id == target_id {
            self.mark_dirty();
            return true;
        }
        for child in &mut self.children {
            if child.mark_dirty_by_id(target_id) {
                self.dirty.paint_dirty = true;
                return true;
            }
        }
        false
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

    pub fn route_window_event(
        &mut self,
        event: &WindowEvent,
        focus: &mut FocusManager,
        scale_factor: f64,
    ) {
        if let WindowEvent::ModifiersChanged(new_mods) = event {
            focus.modifiers = new_mods.state();
        }

        match *event {
            WindowEvent::CursorMoved { position, .. } => {
                let logical_pos: winit::dpi::LogicalPosition<f32> =
                    position.to_logical(scale_factor);
                let pos = glam::vec2(logical_pos.x, logical_pos.y);
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
                if focused_path.is_empty() && key_ev.text.is_none() {
                    return;
                }

                let kind = match key_ev.state {
                    ElementState::Pressed => EventKind::KeyDown {
                        key: key_ev.logical_key.clone(),
                    },
                    ElementState::Released => EventKind::KeyUp {
                        key: key_ev.logical_key.clone(),
                    },
                };
                Self::send_to_path(self, &focused_path, kind, focus);

                if let Some(text) = &key_ev.text {
                    if let Some(ch) = text.chars().next() {
                        Self::send_to_path(self, &focused_path, EventKind::CharInput { ch }, focus);
                    }
                }
            }

            WindowEvent::Focused(false) => {
                focus.blur();
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

        if let Some(new_path) = focus.take_change_request() {
            let old_path = focus.path().to_vec();
            if new_path != old_path {
                if !old_path.is_empty() {
                    Self::send_to_path(self, &old_path, EventKind::FocusOut, focus);
                }
                if !new_path.is_empty() {
                    Self::send_to_path(self, &new_path, EventKind::FocusIn, focus);
                }
                focus.commit_focus_change(new_path);
            }
        }
    }

    fn send_to_path(node: &mut Node, path: &[usize], kind: EventKind, focus: &mut FocusManager) {
        for &phase in &[Phase::Capture, Phase::Target, Phase::Bubble] {
            let mut ctx = EventCtx::new(
                phase,
                focus,
                path,
                Rect::new(Vec2::ZERO, Vec2::ZERO),
                focus.modifiers,
            );
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
