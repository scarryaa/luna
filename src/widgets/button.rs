use std::cell::RefCell;
use std::rc::Rc;

use glam::{Vec2, Vec4, vec2};
use winit::event::{ElementState, MouseButton, WindowEvent};

use super::base::Widget;
use crate::{
    Renderer,
    layout::{Rect, node::Node},
    renderer::{RectId, RenderPrimative, primatives::RectInstance},
    style::tokens::{Colour, Radius, Spacing, Typography},
    windowing::events::{EventCtx, EventKind, Phase},
};

#[derive(Clone)]
pub struct Button {
    pub label: String,
    pub on_click: Rc<RefCell<dyn FnMut()>>,
    pub hovered: bool,
    bg_id: Option<RectId>,
    label_id: Option<usize>,
}

impl Button {
    pub fn label<S: Into<String>>(txt: S) -> Self {
        Self {
            label: txt.into(),
            on_click: Rc::new(RefCell::new(|| {})),
            hovered: false,
            bg_id: None,
            label_id: None,
        }
    }

    pub fn on_click(mut self, handler: impl FnMut() + 'static) -> Self {
        self.on_click = Rc::new(RefCell::new(handler));
        self
    }
}

impl Widget for Button {
    fn measure(&self, _max_width: f32) -> Vec2 {
        let text_w = self.label.len() as f32 * 0.6 * Typography::BODY;
        vec2(
            text_w + Spacing::MD * 2.0,
            Typography::BODY + Spacing::SM * 2.0,
        )
    }

    fn paint(&mut self, _children: &mut [Node], layout: Rect, ren: &mut Renderer) {
        let bg_color = if self.hovered {
            Vec4::from(Colour::PRIMARY_HOVER)
        } else {
            Vec4::from(Colour::PRIMARY)
        };

        let id = *self.bg_id.get_or_insert_with(|| ren.alloc_rect());

        ren.update_rect(
            id,
            RectInstance {
                pos: layout.origin.to_array(),
                size: layout.size.to_array(),
                color: bg_color.to_array(),
                radius: Radius::MD,
                z: 0.0,
                _pad: 0.0,
            },
        );

        let txt_pos = layout.origin + vec2(Spacing::MD, Spacing::SM);

        let text_prim = RenderPrimative::text(
            &self.label,
            txt_pos,
            Vec4::from(Colour::TEXT),
            Typography::BODY,
        );

        if let Some(label_id) = self.label_id {
            ren.update_text(label_id, text_prim);
        } else {
            self.label_id = Some(ren.push_text(text_prim));
        }
    }

    fn hit_test(&self, pt: Vec2, layout: Rect) -> bool {
        layout.contains(pt)
    }

    fn event(&mut self, ctx: &mut EventCtx, ev: &EventKind) {
        match *ev {
            EventKind::PointerDown {
                button: MouseButton::Left,
                ..
            } if ctx.phase == Phase::Target => {
                ctx.focus.request_focus(ctx.path);
                ctx.stop_propagation();
            }
            EventKind::PointerMove { .. } if ctx.phase == Phase::Target => {
                self.hovered = true;
                ctx.stop_propagation();
            }
            EventKind::PointerLeave => {
                self.hovered = false;
            }

            EventKind::PointerUp {
                button: MouseButton::Left,
                ..
            } if ctx.phase == Phase::Target => {
                (self.on_click.borrow_mut())();
                ctx.stop_propagation();
            }
            _ => {}
        }
    }

    fn input(&mut self, event: &WindowEvent) {
        match *event {
            WindowEvent::CursorMoved { .. } => self.hovered = true,
            WindowEvent::CursorLeft { .. } => self.hovered = false,
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => (self.on_click.borrow_mut())(),
            _ => {}
        }
    }
}
