pub mod dirty;
pub mod flexbox;
pub mod grid;
pub mod node;
pub mod rect;

pub use dirty::Dirty;
pub use rect::Rect;
pub struct LayoutNode(pub super::layout::node::Node);

impl LayoutNode {
    pub fn layout(&mut self, max_w: f32) -> glam::Vec2 {
        self.0.layout(max_w)
    }

    pub fn cached_size(&self) -> glam::Vec2 {
        self.0.cached_size
    }

    pub fn set_layout_rect(&mut self, r: Rect) {
        self.0.layout_rect = r
    }

    pub fn parent_origin(&self) -> glam::Vec2 {
        self.0.layout_rect.origin
    }

    pub fn style(&self) -> crate::style::Style {
        self.0.widget.style()
    }
}
