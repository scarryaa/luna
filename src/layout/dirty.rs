#[derive(Default)]
pub struct Dirty {
    pub self_dirty: bool,
    pub child_dirty: bool,
}
