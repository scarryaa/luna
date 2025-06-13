use glam::Vec2;

#[derive(Debug, Copy, Clone, PartialEq)]
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

    pub fn intersects(&self, other: &Rect) -> bool {
        self.origin.x < other.origin.x + other.size.x
            && self.origin.x + self.size.x > other.origin.x
            && self.origin.y < other.origin.y + other.size.y
            && self.origin.y + self.size.y > other.origin.y
    }
}
