use std::time::{Duration, Instant};

use arboard::Clipboard;
use cosmic_text::{Attrs, Buffer, Color, Metrics, Shaping};
use glam::{Vec2, Vec4, vec2};
use winit::keyboard::ModifiersState;
use winit::keyboard::{Key, NamedKey};

use crate::style::Theme;
use crate::{
    Widget,
    layout::{Rect, node::Node},
    renderer::{RectId, Renderer, primatives::RectInstance},
    style::Style,
    windowing::events::{EventCtx, EventKind, Phase},
};

fn is_command_modifier(mods: ModifiersState) -> bool {
    #[cfg(target_os = "macos")]
    {
        mods.super_key()
    }
    #[cfg(not(target_os = "macos"))]
    {
        mods.control_key()
    }
}

#[derive(Clone)]
pub struct TextInput {
    pub value: String,
    pub placeholder: String,
    cursor: usize,
    selection_anchor: usize,
    focused: bool,
    is_dragging: bool,
    scroll_offset: f32,
    last_pos: Vec2,
    click_to_process: Option<Vec2>,

    bg_id: Option<RectId>,
    glyph_rect_ids: Vec<RectId>,
    cursor_id: Option<RectId>,
    selection_id: Option<RectId>,
    last_blink: Instant,
}

impl TextInput {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            value: String::new(),
            placeholder: placeholder.into(),
            cursor: 0,
            selection_anchor: 0,
            focused: false,
            is_dragging: false,
            scroll_offset: 0.0,
            last_pos: Vec2::ZERO,
            click_to_process: None,
            bg_id: None,
            glyph_rect_ids: Vec::new(),
            cursor_id: None,
            selection_id: None,
            last_blink: Instant::now(),
        }
    }

    fn on_input_change(&mut self, ctx: &mut EventCtx) {
        self.last_blink = Instant::now();
        ctx.request_layout();
    }

    fn has_selection(&self) -> bool {
        self.cursor != self.selection_anchor
    }

    fn selection_range(&self) -> (usize, usize) {
        (
            self.cursor.min(self.selection_anchor),
            self.cursor.max(self.selection_anchor),
        )
    }

    fn get_byte_index(&self, char_idx: usize) -> usize {
        self.value
            .char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(self.value.len())
    }

    fn selected_text(&self) -> &str {
        if !self.has_selection() {
            return "";
        }
        let (start_char, end_char) = self.selection_range();
        let start_byte = self.get_byte_index(start_char);
        let end_byte = self.get_byte_index(end_char);
        &self.value[start_byte..end_byte]
    }

    fn delete_selection(&mut self) -> bool {
        if !self.has_selection() {
            return false;
        }
        let (start_char, end_char) = self.selection_range();
        let start_byte = self.get_byte_index(start_char);
        let end_byte = self.get_byte_index(end_char);

        self.value.replace_range(start_byte..end_byte, "");
        self.cursor = start_char;
        self.selection_anchor = start_char;
        true
    }

    fn move_cursor(&mut self, new_pos: usize, keep_selection: bool) {
        self.cursor = new_pos.clamp(0, self.value.chars().count());
        if !keep_selection {
            self.selection_anchor = self.cursor;
        }
    }
}

impl Widget for TextInput {
    fn style(&self) -> Style {
        Style {
            padding: vec2(8.0, 8.0),
            ..Default::default()
        }
    }

    fn measure(&self, _max_width: f32, theme: &Theme) -> Vec2 {
        vec2(100.0, theme.typography.body + theme.spacing.md * 2.0)
    }

    fn paint(&mut self, node: &mut Node, ren: &mut Renderer, theme: &Theme) {
        let layout = node.layout_rect;

        let bg_color = if self.focused {
            Vec4::from(theme.color.primary_hover)
        } else {
            Vec4::from(theme.color.surface)
        };
        let bg_id = *self.bg_id.get_or_insert_with(|| ren.alloc_rect());
        ren.update_rect(
            bg_id,
            RectInstance {
                pos: layout.origin.to_array(),
                size: layout.size.to_array(),
                color: bg_color.to_array(),
                radius: theme.radius.md,
                ..Default::default()
            },
        );

        let padding = theme.spacing.md;
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
            Vec4::from(theme.color.text)
        };

        let selection_instance_data: Option<RectInstance>;
        let cursor_instance_data: Option<RectInstance>;
        let mut visible_glyphs = Vec::new();

        #[derive(Copy, Clone)]
        struct GlyphInfo {
            pos: Vec2,
            size: [f32; 2],
            color: [f32; 4],
        }

        {
            let (font_system, swash_cache) = ren.font_and_swash_cache();
            let metrics = Metrics::new(theme.typography.body, theme.typography.body * 1.2);
            let mut text_buffer = Buffer::new(font_system, metrics);
            let mut buffer_mut = text_buffer.borrow_with(font_system);
            buffer_mut.set_text(text_to_draw, &Attrs::new(), Shaping::Advanced);
            buffer_mut.shape_until_scroll(true);

            if let Some(click_pos) = self.click_to_process.take() {
                let relative_click_x = click_pos.x - content_area.origin.x + self.scroll_offset;
                if let Some(cursor) = buffer_mut.hit(relative_click_x, 0.0) {
                    let char_idx = text_to_draw
                        .char_indices()
                        .take_while(|(i, _)| *i < cursor.index)
                        .count();
                    self.move_cursor(char_idx, self.is_dragging);
                } else {
                    self.move_cursor(text_to_draw.chars().count(), self.is_dragging);
                }
                self.last_blink = Instant::now();
            }

            let cursor_px_offset = buffer_mut.layout_runs().next().map_or(0.0, |run| {
                run.glyphs.iter().take(self.cursor).map(|g| g.w).sum()
            });

            selection_instance_data = if self.focused && self.has_selection() {
                buffer_mut.layout_runs().next().map(|run| {
                    let (start_char, end_char) = self.selection_range();
                    let start_x: f32 = run.glyphs.iter().take(start_char).map(|g| g.w).sum();
                    let end_x: f32 = run.glyphs.iter().take(end_char).map(|g| g.w).sum();

                    let full_selection_rect = Rect::new(
                        vec2(
                            content_area.origin.x + start_x - self.scroll_offset,
                            content_area.origin.y,
                        ),
                        vec2(end_x - start_x, theme.typography.body),
                    );

                    let clipped_rect = content_area.intersection(&full_selection_rect);

                    RectInstance {
                        pos: clipped_rect.origin.to_array(),
                        size: clipped_rect.size.to_array(),
                        color: [0.3, 0.5, 0.9, 0.5],
                        ..Default::default()
                    }
                })
            } else {
                None
            };

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
                if glyph_pos.x + w as f32 >= content_area.origin.x
                    && glyph_pos.x <= content_area.origin.x + content_area.size.x
                {
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

            cursor_instance_data =
                if self.focused && self.last_blink.elapsed() < Duration::from_millis(500) {
                    let cursor_abs_pos =
                        content_area.origin + vec2(cursor_px_offset - self.scroll_offset, 0.0);
                    if cursor_abs_pos.x >= content_area.origin.x
                        && cursor_abs_pos.x <= content_area.origin.x + content_area.size.x
                    {
                        Some(RectInstance {
                            pos: cursor_abs_pos.to_array(),
                            size: [2.0, theme.typography.body],
                            color: Vec4::from(theme.color.text).to_array(),
                            radius: 1.0,
                            ..Default::default()
                        })
                    } else {
                        None
                    }
                } else {
                    None
                };
        }

        let selection_id = *self.selection_id.get_or_insert_with(|| ren.alloc_rect());
        ren.update_rect(selection_id, selection_instance_data.unwrap_or_default());

        let cursor_id = *self.cursor_id.get_or_insert_with(|| ren.alloc_rect());
        ren.update_rect(cursor_id, cursor_instance_data.unwrap_or_default());

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

        if self.focused && self.last_blink.elapsed() > Duration::from_millis(1000) {
            self.last_blink = Instant::now();
        }
    }

    fn event(&mut self, ctx: &mut EventCtx, ev: &EventKind) {
        if !self.focused || ctx.phase != Phase::Target {
            if let EventKind::FocusIn = ev {
                if ctx.phase == Phase::Target {
                    self.focused = true;
                    self.last_blink = Instant::now();
                    ctx.request_layout();
                }
            }
            if let EventKind::PointerDown { .. } = ev {
                if ctx.phase == Phase::Target {
                    ctx.focus.request_focus(ctx.path);
                    self.click_to_process = Some(self.last_pos);
                    self.is_dragging = true;
                    ctx.request_layout();
                }
            }
            return;
        }

        match ev {
            EventKind::FocusOut => {
                self.focused = false;
                self.is_dragging = false;
                ctx.request_layout();
            }
            EventKind::PointerMove { pos, .. } => {
                self.last_pos = *pos;
                if self.is_dragging {
                    self.click_to_process = Some(*pos);
                    ctx.request_layout();
                }
            }
            EventKind::PointerUp { .. } => {
                self.is_dragging = false;
            }
            EventKind::CharInput { ch } => {
                let mods = ctx.modifiers;
                if !ch.is_control() && !is_command_modifier(mods) && !mods.alt_key() {
                    self.delete_selection();
                    let byte_idx = self.get_byte_index(self.cursor);
                    self.value.insert(byte_idx, *ch);
                    self.move_cursor(self.cursor + 1, false);
                    self.on_input_change(ctx);
                }
            }
            EventKind::KeyDown { key } => {
                let keep_selection = ctx.modifiers.shift_key();

                if is_command_modifier(ctx.modifiers) {
                    match key {
                        Key::Character(s) if s == "a" => {
                            self.selection_anchor = 0;
                            self.cursor = self.value.chars().count();
                        }
                        Key::Character(s) if s == "c" => {
                            if let Ok(mut clip) = Clipboard::new() {
                                clip.set_text(self.selected_text()).ok();
                            }
                        }
                        Key::Character(s) if s == "x" => {
                            if let Ok(mut clip) = Clipboard::new() {
                                clip.set_text(self.selected_text()).ok();
                            }
                            self.delete_selection();
                        }
                        Key::Character(s) if s == "v" => {
                            if let Ok(mut clip) = Clipboard::new() {
                                if let Ok(text) = clip.get_text() {
                                    let sanitized_text = text.replace('\r', "").replace('\n', " ");

                                    self.delete_selection();
                                    let byte_idx = self.get_byte_index(self.cursor);
                                    self.value.insert_str(byte_idx, &sanitized_text);
                                    self.move_cursor(
                                        self.cursor + sanitized_text.chars().count(),
                                        false,
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                } else {
                    match key {
                        Key::Named(NamedKey::Backspace) => {
                            if !self.delete_selection() && self.cursor > 0 {
                                let new_cursor_pos = self.cursor - 1;
                                let byte_idx_to_remove = self.get_byte_index(new_cursor_pos);
                                self.value.remove(byte_idx_to_remove);
                                self.move_cursor(new_cursor_pos, false);
                            }
                        }
                        Key::Named(NamedKey::Delete) => {
                            if !self.delete_selection() && self.cursor < self.value.chars().count()
                            {
                                let byte_idx = self.get_byte_index(self.cursor);
                                self.value.remove(byte_idx);
                                self.move_cursor(self.cursor, false);
                            }
                        }
                        Key::Named(NamedKey::ArrowLeft) => {
                            self.move_cursor(self.cursor.saturating_sub(1), keep_selection);
                        }
                        Key::Named(NamedKey::ArrowRight) => {
                            self.move_cursor(self.cursor + 1, keep_selection);
                        }
                        Key::Named(NamedKey::Home) => self.move_cursor(0, keep_selection),
                        Key::Named(NamedKey::End) => {
                            self.move_cursor(self.value.chars().count(), keep_selection)
                        }
                        _ => return,
                    }
                }
                self.on_input_change(ctx);
            }
            _ => {}
        }
    }
}
