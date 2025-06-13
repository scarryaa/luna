#[derive(Default, Debug, Copy, Clone)]
pub struct Dirty {
    pub self_dirty: bool,  // needs a new measurement (size)
    pub child_dirty: bool, // some descendant is self_dirty
    pub paint_dirty: bool, // visual representation changed
}
