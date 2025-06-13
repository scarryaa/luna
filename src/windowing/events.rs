use glam::Vec2;
use winit::keyboard::ModifiersState;
use winit::{event::MouseButton, keyboard::Key};

#[derive(Clone, Debug)]
pub enum EventKind {
    PointerDown { button: MouseButton, pos: Vec2 },
    PointerUp { button: MouseButton, pos: Vec2 },
    PointerMove { pos: Vec2 },
    PointerLeave,
    Wheel { delta: Vec2 },

    KeyDown { key: Key },
    KeyUp { key: Key },
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
    pub node_layout: crate::layout::Rect,
    stopped: bool,
    default_prevented: bool,
    pub layout_requested: bool,
    pub modifiers: ModifiersState,
}

impl<'a> EventCtx<'a> {
    pub fn request_layout(&mut self) {
        self.layout_requested = true;
    }

    pub fn stop_propagation(&mut self) {
        self.stopped = true;
    }

    pub fn prevent_default(&mut self) {
        self.default_prevented = true;
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    pub fn new(
        phase: Phase,
        focus: &'a mut FocusManager,
        path: &'a [usize],
        node_layout: crate::layout::Rect,
        modifiers: ModifiersState,
    ) -> Self {
        Self {
            phase,
            focus,
            path,
            node_layout,
            stopped: false,
            default_prevented: false,
            layout_requested: false,
            modifiers,
        }
    }
}

#[derive(Default)]
pub struct FocusManager {
    focused_path: Vec<usize>,
    change_request: Option<Vec<usize>>,
    pub modifiers: ModifiersState,
}

impl FocusManager {
    pub fn request_focus(&mut self, path: &[usize]) {
        self.change_request = Some(path.to_vec());
    }

    pub fn blur(&mut self) {
        self.change_request = Some(Vec::new());
    }

    pub fn path(&self) -> &[usize] {
        &self.focused_path
    }

    #[allow(dead_code)]
    pub fn has_focus(&self, path: &[usize]) -> bool {
        path == self.focused_path
    }

    pub(crate) fn take_change_request(&mut self) -> Option<Vec<usize>> {
        self.change_request.take()
    }

    pub(crate) fn commit_focus_change(&mut self, new_path: Vec<usize>) {
        self.focused_path = new_path;
    }
}
