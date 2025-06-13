use std::time::{Duration, Instant};

use cosmic_text::{Attrs, Buffer, Color, Metrics, Shaping};
use glam::{Vec2, Vec4, vec2};
use winit::keyboard::{Key, NamedKey};

use crate::{
    Widget,
    layout::{Rect, node::Node},
    renderer::{RectId, Renderer, primatives::RectInstance},
    style::{
        Style,
        tokens::{Colour, Radius, Spacing, Typography},
    },
    windowing::events::{EventCtx, EventKind, Phase},
};

#[derive(Clone)]
pub struct TextInput {
    pub value: String,
    pub placeholder: String,
    cursor_pos: usize,
    focused: bool,
    scroll_offset: f32,
    last_pos: Vec2,
    click_to_process: Option<Vec2>,

    bg_id: Option<RectId>,
    glyph_rect_ids: Vec<RectId>,
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
            scroll_offset: 0.0,
            last_pos: Vec2::ZERO,
            click_to_process: None,
            bg_id: None,
            glyph_rect_ids: Vec::new(),
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
    fn style(&self) -> Style {
        Style {
            padding: vec2(Spacing::MD, Spacing::MD),
            ..Default::default()
        }
    }

    fn measure(&self, _max_width: f32) -> Vec2 {
        vec2(100.0, Typography::BODY + Spacing::MD * 2.0)
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
                ..Default::default()
            },
        );

        let padding = Spacing::MD;
        let content_area = Rect::new(
            layout.origin + padding,
            layout.size - vec2(padding * 2.0, padding * 2.0),
        );
        let text_to_draw = if self.value.is_empty() && !self.focused {
            &self.placeholder
        } else {
            &self.value
        };
        let text_color = if self.value.is_empty() && !self.focused {
            Vec4::new(0.5, 0.5, 0.5, 1.0)
        } else {
            Vec4::from(Colour::TEXT)
        };

        #[derive(Copy, Clone)]
        struct GlyphInfo {
            pos: Vec2,
            size: [f32; 2],
            color: [f32; 4],
        }
        let mut visible_glyphs: Vec<GlyphInfo> = Vec::new();

        let cursor_px_offset;
        {
            let (font_system, swash_cache) = ren.font_and_swash_cache();

            let metrics = Metrics::new(Typography::BODY, Typography::BODY * 1.2);
            let mut text_buffer = Buffer::new(font_system, metrics);
            let mut buffer_mut = text_buffer.borrow_with(font_system);
            buffer_mut.set_text(text_to_draw, &Attrs::new(), Shaping::Advanced);
            buffer_mut.shape_until_scroll(true);

            if let Some(click_pos) = self.click_to_process.take() {
                let relative_click_x = click_pos.x - content_area.origin.x + self.scroll_offset;
                if let Some(cursor) = buffer_mut.hit(relative_click_x, 0.0) {
                    self.cursor_pos = text_to_draw
                        .char_indices()
                        .take_while(|(i, _)| *i < cursor.index)
                        .count();
                } else {
                    self.cursor_pos = text_to_draw.chars().count();
                }
                self.last_blink = Instant::now();
            }

            cursor_px_offset = buffer_mut.layout_runs().next().map_or(0.0, |run| {
                run.glyphs.iter().take(self.cursor_pos).map(|g| g.w).sum()
            });

            if cursor_px_offset < self.scroll_offset {
                self.scroll_offset = cursor_px_offset;
            } else if cursor_px_offset > self.scroll_offset + content_area.size.x {
                self.scroll_offset = cursor_px_offset - content_area.size.x;
            }
            let total_text_width = buffer_mut.layout_runs().next().map_or(0.0, |r| r.line_w);
            let max_scroll = (total_text_width - content_area.size.x).max(0.0);
            self.scroll_offset = self.scroll_offset.clamp(0.0, max_scroll);

            let fg = Color::rgba(
                (text_color.x * 255.0) as u8,
                (text_color.y * 255.0) as u8,
                (text_color.z * 255.0) as u8,
                (text_color.w * 255.0) as u8,
            );

            buffer_mut.draw(swash_cache, fg, |x, y, w, h, color| {
                let glyph_pos =
                    vec2(x as f32, y as f32) + content_area.origin - vec2(self.scroll_offset, 0.0);
                let glyph_rect = Rect::new(glyph_pos, vec2(w as f32, h as f32));
                if content_area.intersects(&glyph_rect) {
                    visible_glyphs.push(GlyphInfo {
                        pos: glyph_pos,
                        size: [w as f32, h as f32],
                        color: [
                            color.r() as f32 / 255.0,
                            color.g() as f32 / 255.0,
                            color.b() as f32 / 255.0,
                            color.a() as f32 / 255.0,
                        ],
                    });
                }
            });
        }

        let num_visible = visible_glyphs.len();
        if num_visible > self.glyph_rect_ids.len() {
            for _ in 0..(num_visible - self.glyph_rect_ids.len()) {
                self.glyph_rect_ids.push(ren.alloc_rect());
            }
        }
        for i in 0..num_visible {
            let glyph = &visible_glyphs[i];
            ren.update_rect(
                self.glyph_rect_ids[i],
                RectInstance {
                    pos: glyph.pos.to_array(),
                    size: glyph.size,
                    color: glyph.color,
                    ..Default::default()
                },
            );
        }
        for i in num_visible..self.glyph_rect_ids.len() {
            ren.update_rect(self.glyph_rect_ids[i], RectInstance::default());
        }
        self.glyph_rect_ids.truncate(num_visible);

        let cursor_id = *self.cursor_id.get_or_insert_with(|| ren.alloc_rect());
        if self.focused && self.last_blink.elapsed() < Duration::from_millis(500) {
            let cursor_abs_pos =
                content_area.origin + vec2(cursor_px_offset - self.scroll_offset, 0.0);
            ren.update_rect(
                cursor_id,
                RectInstance {
                    pos: cursor_abs_pos.to_array(),
                    size: [2.0, Typography::BODY].into(),
                    color: Vec4::from(Colour::TEXT).to_array(),
                    radius: 1.0,
                    ..Default::default()
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
            EventKind::PointerMove { pos, .. } => self.last_pos = *pos,
            EventKind::PointerDown { .. } if ctx.phase == Phase::Target => {
                ctx.focus.request_focus(ctx.path);
                self.click_to_process = Some(self.last_pos);
                ctx.request_layout();
            }
            EventKind::CharInput { ch } if self.focused && ctx.phase == Phase::Target => {
                if !ch.is_control() {
                    let (byte_idx, _) = self
                        .value
                        .char_indices()
                        .nth(self.cursor_pos)
                        .map_or((self.value.len(), '\0'), |(i, c)| (i, c));
                    self.value.insert(byte_idx, *ch);
                    self.cursor_pos += 1;
                    self.on_input_change(ctx);
                }
            }
            EventKind::KeyDown { key } if self.focused && ctx.phase == Phase::Target => match key {
                Key::Named(NamedKey::Backspace) => {
                    if !self.value.is_empty() && self.cursor_pos > 0 {
                        let (byte_idx, _) =
                            self.value.char_indices().nth(self.cursor_pos - 1).unwrap();
                        self.value.remove(byte_idx);
                        self.cursor_pos -= 1;
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
