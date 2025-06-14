pub mod app;
pub mod debug;
pub mod layout;
pub mod renderer;
pub mod signals;
pub mod style;
pub mod text;
pub mod widgets;
pub mod windowing;

pub use anyhow::Result;
pub use app::App;
pub use layout::LayoutNode;
pub use renderer::Renderer;
pub use style::Style;
pub use widgets::{Button, Canvas, Checkbox, Element, Image, Text, TextInput, Widget};
pub use windowing::{Window, WindowBuilder};

pub use glam::{Mat4, Vec2, Vec3, Vec4, vec2};
pub use winit::event::{Event, MouseButton, WindowEvent};
pub use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};

pub use style::{Align, Display, FlexDir, Justify, Theme};

pub fn init_logging() {
    env_logger::init();
}
