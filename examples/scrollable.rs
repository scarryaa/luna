use glam::{Vec2, vec2};
use luna::layout::Rect;
use luna::widgets::{Button, Column, Scrollable};
use luna::*;
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

fn main() -> Result<()> {
    init_logging();

    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Scrollable Demo")
            .with_inner_size(winit::dpi::LogicalSize::new(240, 480))
            .build(&event_loop)?,
    );
    let cloned_window = window.clone();

    let mut renderer =
        pollster::block_on(Renderer::new(&cloned_window, window.scale_factor() as f32))?;

    let many_buttons: Vec<Box<dyn Widget>> = (0..100)
        .map(|i| Box::new(Button::label(format!("Button #{i}"))) as Box<dyn Widget>)
        .collect();

    let content = Column {
        spacing: 8.0,
        children: many_buttons,
    };

    let root_widget = Scrollable::new(content);
    let mut root = layout::node::Node::new(
        Box::new(root_widget),
        Rect::new(vec2(0.0, 0.0), vec2(240.0, 480.0)),
        &mut widgets::BuildCtx,
    );

    let mut win_size = window.inner_size();
    let mut focus_manager = windowing::events::FocusManager::default();
    window.request_redraw();

    let _ = event_loop.run(move |event, elwt| match &event {
        Event::WindowEvent { window_id, event } if *window_id == window.id() => match event {
            WindowEvent::RedrawRequested => {
                renderer.begin_frame();
                root.layout(win_size.width as f32);
                root.collect(&mut renderer);
                renderer.end_frame().ok();
            }
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::Resized(new_size) => {
                win_size = *new_size;
                renderer.resize(*new_size);
                root.set_rect(Rect::new(
                    Vec2::ZERO,
                    vec2(new_size.width as f32, new_size.height as f32),
                ));
                root.mark_dirty();
                window.request_redraw();
            }
            _ => {
                root.route_window_event(event, &mut focus_manager, window.scale_factor());
                window.request_redraw();
            }
        },
        _ => {}
    });

    Ok(())
}
