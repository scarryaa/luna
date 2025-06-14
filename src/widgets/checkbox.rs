use glam::{Vec2, Vec4, vec2};
use winit::event::MouseButton;
use winit::keyboard::{Key, NamedKey};

use crate::{
    Widget,
    layout::node::Node,
    renderer::{RectId, Renderer, primatives::RectInstance},
    signals::{ReadSignal, WriteSignal, create_signal},
    style::Theme,
    windowing::events::{EventCtx, EventKind},
};

#[derive(Clone)]
pub struct Checkbox {
    pub label: ReadSignal<String>,
    pub state: (ReadSignal<bool>, WriteSignal<bool>),

    hovered: bool,
    focused: bool,
    pressed: bool,

    border_id: Option<RectId>,
    fill_id: Option<RectId>,
    focus_ring_id: Option<RectId>,
}

impl Checkbox {
    pub fn new(
        label: impl Into<ReadSignal<String>>,
        state: (ReadSignal<bool>, WriteSignal<bool>),
    ) -> Self {
        Self {
            label: label.into(),
            state,
            hovered: false,
            focused: false,
            pressed: false,
            border_id: None,
            fill_id: None,
            focus_ring_id: None,
        }
    }

    pub fn new_with_label(label: &str) -> Self {
        let (read, write) = create_signal(false);
        Self::new(label.to_string(), (read, write))
    }
}

impl Widget for Checkbox {
    fn measure(
        &self,
        _max_width: f32,
        theme: &Theme,
        font_system: &mut cosmic_text::FontSystem,
    ) -> Vec2 {
        let box_size = theme.typography.body;
        let spacing = theme.spacing.sm;

        let text_w = {
            let mut buffer = cosmic_text::Buffer::new(
                font_system,
                cosmic_text::Metrics::new(theme.typography.body, theme.typography.body * 1.2),
            );
            let mut buffer_mut = buffer.borrow_with(font_system);
            buffer_mut.set_text(
                &self.label.get(),
                &cosmic_text::Attrs::new(),
                cosmic_text::Shaping::Advanced,
            );
            buffer_mut.shape_until_scroll(true);
            buffer_mut
                .layout_runs()
                .next()
                .map_or(0.0, |run| run.line_w)
        };

        vec2(box_size + spacing + text_w, box_size)
    }

    fn event(&mut self, ctx: &mut EventCtx, ev: &EventKind) {
        match *ev {
            EventKind::PointerDown {
                button: MouseButton::Left,
                ..
            } => {
                self.pressed = true;
                self.state.1.update(|v| *v = !*v);
                ctx.focus.request_focus(ctx.path);
            }
            EventKind::PointerUp {
                button: MouseButton::Left,
                ..
            } => {
                self.pressed = false;
            }
            EventKind::PointerMove { .. } => {
                self.hovered = true;
            }
            EventKind::PointerLeave => {
                self.hovered = false;
                self.pressed = false;
            }
            EventKind::KeyDown {
                key: Key::Named(NamedKey::Space),
                ..
            } if self.focused => {
                self.state.1.update(|v| *v = !*v);
            }
            EventKind::FocusIn => {
                self.focused = true;
            }
            EventKind::FocusOut => {
                self.focused = false;
            }
            _ => {}
        }
    }

    fn paint(&mut self, node: &mut Node, ren: &mut Renderer, theme: &Theme) {
        let layout = node.layout_rect;
        let is_checked = self.state.0.get();
        let box_size = theme.typography.body;

        let focus_ring_id = *self.focus_ring_id.get_or_insert_with(|| ren.alloc_rect());
        if self.focused {
            let offset = 2.0;
            let ring_pos = layout.origin - offset;
            let ring_size = box_size + offset * 2.0;
            let ring_color = Vec4::new(
                theme.color.primary[0],
                theme.color.primary[1],
                theme.color.primary[2],
                0.5,
            );

            ren.update_rect(
                focus_ring_id,
                RectInstance {
                    pos: ring_pos.to_array(),
                    size: [ring_size, ring_size],
                    color: ring_color.to_array(),
                    radius: theme.radius.sm + offset,
                    ..Default::default()
                },
            );
        } else {
            ren.update_rect(focus_ring_id, RectInstance::default());
        }

        let border_color = if self.hovered {
            Vec4::from(theme.color.primary) * 0.9
        } else {
            Vec4::from(theme.color.text) * Vec4::new(1.0, 1.0, 1.0, 0.5)
        };

        let border_id = *self.border_id.get_or_insert_with(|| ren.alloc_rect());
        ren.update_rect(
            border_id,
            RectInstance {
                pos: layout.origin.to_array(),
                size: [box_size, box_size],
                color: border_color.to_array(),
                radius: theme.radius.sm,
                ..Default::default()
            },
        );

        let fill_id = *self.fill_id.get_or_insert_with(|| ren.alloc_rect());
        if is_checked {
            let border_width = 2.0;
            let fill_size = (box_size - (border_width * 2.0)).max(0.0);
            let fill_pos = layout.origin + border_width;

            let fill_color = if self.pressed {
                Vec4::from(theme.color.primary_hover)
            } else {
                Vec4::from(theme.color.primary)
            };

            ren.update_rect(
                fill_id,
                RectInstance {
                    pos: fill_pos.to_array(),
                    size: [fill_size, fill_size],
                    color: fill_color.to_array(),
                    radius: (theme.radius.sm - (border_width / 2.0)).max(0.0),
                    ..Default::default()
                },
            );
        } else {
            ren.update_rect(fill_id, RectInstance::default());
        }

        let label_pos = layout.origin + vec2(box_size + theme.spacing.sm, 0.0);
        ren.draw_text(
            &self.label.get(),
            label_pos,
            theme.color.text.into(),
            theme.typography.body,
        );
    }
}
