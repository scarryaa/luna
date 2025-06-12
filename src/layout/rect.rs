use glam::Vec2;

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub origin: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub fn new(origin: Vec2, size: Vec2) -> Self {
        Self { origin, size }
    }

    pub fn contains(&self, p: Vec2) -> bool {
        let min = self.origin;
        let max = self.origin + self.size;
        p.x >= min.x && p.x <= max.x && p.y >= min.y && p.y <= max.y
    }
}
