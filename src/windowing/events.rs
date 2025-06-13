use glam::Vec2;
use winit::{event::MouseButton, keyboard::KeyCode};

#[derive(Clone, Debug)]
pub enum EventKind {
    PointerDown { button: MouseButton, pos: Vec2 },
    PointerUp { button: MouseButton, pos: Vec2 },
    PointerMove { pos: Vec2 },
    PointerLeave,
    Wheel { delta: Vec2 },

    KeyDown { key: KeyCode },
    KeyUp { key: KeyCode },
    CharInput { ch: char },

    FocusIn,
    FocusOut,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Phase {
    Capture,
    Target,
    Bubble,
}

pub struct EventCtx<'a> {
    pub phase: Phase,
    pub focus: &'a mut FocusManager,
    pub path: &'a [usize],
    stopped: bool,
    default_prevented: bool,
}

impl<'a> EventCtx<'a> {
    pub fn stop_propagation(&mut self) {
        self.stopped = true;
    }

    pub fn prevent_default(&mut self) {
        self.default_prevented = true;
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    pub fn new(phase: Phase, focus: &'a mut FocusManager, path: &'a [usize]) -> Self {
        Self {
            phase,
            focus,
            path,
            stopped: false,
            default_prevented: false,
        }
    }
}

#[derive(Default)]
pub struct FocusManager {
    focused_path: Vec<usize>,
}

impl FocusManager {
    pub fn request_focus(&mut self, path: &[usize]) {
        self.focused_path = path.to_vec();
    }

    pub fn blur(&mut self) {
        self.focused_path.clear();
    }

    pub fn path(&self) -> &[usize] {
        &self.focused_path
    }

    pub fn has_focus(&self, path: &[usize]) -> bool {
        path == self.focused_path
    }
}
