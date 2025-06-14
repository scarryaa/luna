use crate::layout::node::Node;
use crate::style::Theme;
use std::rc::Rc;

use glam::{Vec2, Vec4, vec2};
use winit::event::MouseButton;

use super::base::Widget;
use crate::signals::{ReadSignal, create_signal};
use crate::{
    Renderer,
    layout::Rect,
    renderer::{RenderPrimative, primatives::RectInstance},
    windowing::events::{EventCtx, EventKind, Phase},
};

#[derive(Clone)]
pub struct Button {
    pub label: ReadSignal<String>,
    pub on_click: Rc<dyn Fn() + 'static>,
    pub hovered: bool,
    bg_id: Option<usize>,
    label_id: Option<usize>,
}

impl Button {
    pub fn new(label: impl Into<ReadSignal<String>>) -> Self {
        Self {
            label: label.into(),
            on_click: Rc::new(|| {}),
            hovered: false,
            bg_id: None,
            label_id: None,
        }
    }

    pub fn label(txt: &str) -> Self {
        let (read_label, _) = create_signal(txt.to_string());
        Self::new(read_label)
    }

    pub fn on_click(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_click = Rc::new(handler);
        self
    }
}

impl Widget for Button {
    fn measure(&self, _max_width: f32, theme: &Theme) -> Vec2 {
        let text = self.label.get();
        // TODO measure this accurately
        let text_w = text.len() as f32 * 0.6 * theme.typography.body;
        vec2(
            text_w + theme.spacing.md * 2.0,
            theme.typography.body + theme.spacing.sm * 2.0,
        )
    }

    fn paint(&mut self, node: &mut Node, ren: &mut Renderer, theme: &Theme) {
        let layout = node.layout_rect;

        let bg_color = if self.hovered {
            Vec4::from(theme.color.primary_hover)
        } else {
            Vec4::from(theme.color.primary)
        };

        let id = *self.bg_id.get_or_insert_with(|| ren.alloc_rect());

        ren.update_rect(
            id,
            RectInstance {
                pos: layout.origin.to_array(),
                size: layout.size.to_array(),
                color: bg_color.to_array(),
                radius: theme.radius.md,
                z: 0.0,
                _pad: 0.0,
            },
        );

        let txt_pos = layout.origin + vec2(theme.spacing.md, theme.spacing.sm);
        let text_prim = RenderPrimative::text(
            &self.label.get(),
            txt_pos,
            Vec4::from(theme.color.text),
            theme.typography.body,
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
                (self.on_click)();
                ctx.stop_propagation();
            }
            _ => {}
        }
    }
}
