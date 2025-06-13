pub mod events;

pub struct Window {
    pub root: crate::layout::node::Node,
    pub focus: events::FocusManager,
}

pub struct WindowBuilder;
