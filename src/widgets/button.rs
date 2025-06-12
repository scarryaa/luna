use std::cell::RefCell;
use std::rc::Rc;

use glam::{Vec2, vec2, vec4};
use winit::event::{ElementState, MouseButton, WindowEvent};

use super::base::Widget;
use crate::{layout::Rect, renderer::RenderPrimative};

#[derive(Clone)]
pub struct Button {
    pub label: String,
    pub on_click: Rc<RefCell<dyn FnMut()>>,
    pub hovered: bool,
}

impl Widget for Button {
    fn paint(&self, layout: Rect, out: &mut Vec<RenderPrimative>) {
        let bg = if self.hovered {
            vec4(0.3, 0.5, 0.9, 1.0)
        } else {
            vec4(0.2, 0.2, 0.2, 1.0)
        };

        out.push(RenderPrimative::rectangle(layout.origin, layout.size, bg));

        let txt_pos = layout.origin + vec2(8.0, 4.0);
        out.push(RenderPrimative::text(
            &self.label,
            txt_pos,
            vec4(1.0, 1.0, 1.0, 1.0),
            18.0,
        ));
    }

    fn hit_test(&self, pt: Vec2, layout: Rect) -> bool {
        layout.contains(pt)
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
