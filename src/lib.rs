pub mod app;
pub mod debug;
pub mod layout;
pub mod renderer;
pub mod style;
pub mod text;
pub mod widgets;
pub mod windowing;

pub use app::{App, AppBuilder};
pub use layout::LayoutNode;
pub use renderer::Renderer;
pub use style::Style;
pub use widgets::{Button, Text, Widget};
pub use windowing::{Window, WindowBuilder};

pub use glam::{Mat4, Vec2, Vec3, Vec4};
pub use winit::event::{Event, MouseButton, WindowEvent};
pub use winit::keyboard::{KeyCode, PhysicalKey};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn init_logging() {
    env_logger::init();
}
