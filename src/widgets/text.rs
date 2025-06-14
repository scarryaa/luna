use cosmic_text::{Attrs, Buffer, Metrics, Shaping};
use glam::{Vec2, Vec4, vec2};

use crate::{Widget, layout::node::Node, renderer::Renderer, signals::ReadSignal, style::Theme};

#[derive(Clone)]
pub struct Text {
    pub content: ReadSignal<String>,
    pub color: Option<Vec4>,
    pub size: Option<f32>,
}

impl Text {
    pub fn new(content: impl Into<ReadSignal<String>>) -> Self {
        Self {
            content: content.into(),
            color: None,
            size: None,
        }
    }

    pub fn from_str(content: &str) -> Self {
        Self::new(content.to_string())
    }

    pub fn with_color(mut self, color: impl Into<Vec4>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }
}

impl Widget for Text {
    fn measure(
        &self,
        _max_width: f32,
        theme: &Theme,
        font_system: &mut cosmic_text::FontSystem,
    ) -> Vec2 {
        let content_str = self.content.get();
        let size = self.size.unwrap_or(theme.typography.body);
        let metrics = Metrics::new(size, size * 1.2);

        let mut text_buffer = Buffer::new(font_system, metrics);
        let mut buffer_mut = text_buffer.borrow_with(font_system);

        buffer_mut.set_text(&content_str, &Attrs::new(), Shaping::Advanced);
        buffer_mut.shape_until_scroll(true);

        let text_width = buffer_mut
            .layout_runs()
            .next()
            .map_or(0.0, |run| run.line_w);

        vec2(text_width, size)
    }

    fn paint(&mut self, node: &mut Node, ren: &mut Renderer, theme: &Theme) {
        let color = self.color.unwrap_or_else(|| theme.color.text.into());
        let size = self.size.unwrap_or(theme.typography.body);

        let text_width = {
            let mut text_buffer = cosmic_text::Buffer::new(
                ren.font_system(),
                cosmic_text::Metrics::new(size, size * 1.2),
            );
            let mut buffer_mut = text_buffer.borrow_with(ren.font_system());
            buffer_mut.set_text(
                &self.content.get(),
                &cosmic_text::Attrs::new(),
                cosmic_text::Shaping::Advanced,
            );
            buffer_mut.shape_until_scroll(true);
            buffer_mut
                .layout_runs()
                .next()
                .map_or(0.0, |run| run.line_w)
        };

        let offset_x = (node.layout_rect.size.x - text_width) / 2.0;
        let draw_pos = node.layout_rect.origin + vec2(offset_x, 0.0);

        ren.draw_text(&self.content.get(), draw_pos, color, size);
    }
}
