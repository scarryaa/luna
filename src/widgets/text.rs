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
    fn measure(&self, _max_width: f32, theme: &Theme) -> Vec2 {
        let content_str = self.content.get();
        let size = self.size.unwrap_or(theme.typography.body);
        vec2(content_str.len() as f32 * size * 0.6, size)
    }

    fn paint(&mut self, node: &mut Node, ren: &mut Renderer, theme: &Theme) {
        let color = self.color.unwrap_or_else(|| theme.color.text.into());
        let size = self.size.unwrap_or(theme.typography.body);

        ren.draw_text(&self.content.get(), node.layout_rect.origin, color, size);
    }
}
