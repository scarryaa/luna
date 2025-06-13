use crate::Widget;

#[macro_export]
macro_rules! dbg_ev {
    ($ev:expr) => {{
        match $ev {
            EventKind::PointerDown { .. } => "PointerDown",
            EventKind::PointerUp { .. } => "PointerUp",
            EventKind::PointerMove { .. } => "PointerMove",
            EventKind::PointerLeave => "PointerLeave",
            EventKind::KeyDown { .. } => "KeyDown",
            EventKind::KeyUp { .. } => "KeyUp",
            _ => "…",
        }
    }};
}

pub fn widget_type(_w: &Box<dyn Widget>) -> &'static str {
    std::any::type_name::<Box<dyn Widget>>()
        .rsplit_once("::")
        .map(|(_, t)| t)
        .unwrap_or("Widget")
}
