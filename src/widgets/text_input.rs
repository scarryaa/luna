use std::time::{Duration, Instant};

use glam::{Vec2, Vec4, vec2};
use winit::keyboard::{Key, NamedKey};

use crate::{
    Renderer, Widget,
    layout::{Rect, node::Node},
    renderer::{RectId, RenderPrimative, primatives::RectInstance},
    style::tokens::{Colour, Radius, Spacing, Typography},
    windowing::events::{EventCtx, EventKind, Phase},
};

#[derive(Clone)]
pub struct TextInput {
    pub value: String,
    pub placeholder: String,
    cursor_pos: usize, // character index
    focused: bool,
    bg_id: Option<RectId>,
    text_id: Option<usize>,
    cursor_id: Option<RectId>,
    last_blink: Instant,
}

impl TextInput {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            value: String::new(),
            placeholder: placeholder.into(),
            cursor_pos: 0,
            focused: false,
            bg_id: None,
            text_id: None,
            cursor_id: None,
            last_blink: Instant::now(),
        }
    }

    fn on_input_change(&mut self, ctx: &mut EventCtx) {
        self.last_blink = Instant::now();
        ctx.request_layout();
    }
}

impl Widget for TextInput {
    fn measure(&self, _max_width: f32) -> Vec2 {
        let char_count = if self.value.is_empty() {
            self.placeholder.chars().count()
        } else {
            self.value.chars().count()
        };

        let text_width = char_count as f32 * Typography::BODY * 0.55;

        vec2(
            text_width + Spacing::MD * 2.0,
            Typography::BODY + Spacing::MD * 2.0,
        )
    }

    fn paint(&mut self, _children: &mut [Node], layout: Rect, ren: &mut Renderer) {
        let bg_color = if self.focused {
            Vec4::from(Colour::PRIMARY_HOVER)
        } else {
            Vec4::from(Colour::SURFACE)
        };
        let bg_id = *self.bg_id.get_or_insert_with(|| ren.alloc_rect());
        ren.update_rect(
            bg_id,
            RectInstance {
                pos: layout.origin.to_array(),
                size: layout.size.to_array(),
                color: bg_color.to_array(),
                radius: Radius::MD,
                z: 0.0,
                _pad: 0.0,
            },
        );

        let text_to_draw = if self.value.is_empty() {
            &self.placeholder
        } else {
            &self.value
        };
        let text_color = if self.value.is_empty() && !self.focused {
            Vec4::new(0.5, 0.5, 0.5, 1.0)
        } else {
            Vec4::from(Colour::TEXT)
        };
        let text_pos = layout.origin + vec2(Spacing::MD, Spacing::MD);
        let text_prim = RenderPrimative::text(text_to_draw, text_pos, text_color, Typography::BODY);
        if let Some(text_id) = self.text_id {
            ren.update_text(text_id, text_prim);
        } else {
            self.text_id = Some(ren.push_text(text_prim));
        }

        let cursor_id = *self.cursor_id.get_or_insert_with(|| ren.alloc_rect());
        if self.focused && self.last_blink.elapsed() < Duration::from_millis(500) {
            let char_width_approx = Typography::BODY * 0.55;
            let cursor_offset_x = self.cursor_pos as f32 * char_width_approx;
            let cursor_pos = text_pos + vec2(cursor_offset_x, -2.0);
            ren.update_rect(
                cursor_id,
                RectInstance {
                    pos: cursor_pos.to_array(),
                    size: [2.0, Typography::BODY + 4.0].into(),
                    color: Vec4::from(Colour::TEXT).to_array(),
                    radius: 1.0,
                    z: 0.0,
                    _pad: 0.0,
                },
            );
        } else {
            ren.update_rect(cursor_id, RectInstance::default());
        }

        if self.focused && self.last_blink.elapsed() > Duration::from_millis(1000) {
            self.last_blink = Instant::now();
        }
    }

    fn event(&mut self, ctx: &mut EventCtx, ev: &EventKind) {
        match ev {
            EventKind::FocusIn => {
                self.focused = true;
                self.last_blink = Instant::now();
                ctx.request_layout();
            }
            EventKind::FocusOut => {
                self.focused = false;
                ctx.request_layout();
            }
            EventKind::PointerDown { .. } if ctx.phase == Phase::Target => {
                ctx.focus.request_focus(ctx.path);
            }
            EventKind::CharInput { ch } if self.focused && ctx.phase == Phase::Target => {
                let byte_idx = self
                    .value
                    .char_indices()
                    .map(|(i, _)| i)
                    .nth(self.cursor_pos)
                    .unwrap_or(self.value.len());
                self.value.insert(byte_idx, *ch);
                self.cursor_pos += 1;
                self.on_input_change(ctx);
            }
            EventKind::KeyDown { key } if self.focused && ctx.phase == Phase::Target => match key {
                Key::Named(NamedKey::Backspace) => {
                    if self.cursor_pos > 0 {
                        self.cursor_pos -= 1;
                        let (byte_idx, _) = self.value.char_indices().nth(self.cursor_pos).unwrap();
                        self.value.remove(byte_idx);
                        self.on_input_change(ctx);
                    }
                }
                Key::Named(NamedKey::ArrowLeft) => {
                    if self.cursor_pos > 0 {
                        self.cursor_pos -= 1;
                        self.on_input_change(ctx);
                    }
                }
                Key::Named(NamedKey::ArrowRight) => {
                    if self.cursor_pos < self.value.chars().count() {
                        self.cursor_pos += 1;
                        self.on_input_change(ctx);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}
