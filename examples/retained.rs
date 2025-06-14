use glam::vec2;
use luna::{
    layout::node::Node,
    signals::{self, create_signal},
    windowing::events::FocusManager,
};
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use luna::{
    Button, Result,
    layout::Rect,
    renderer::Renderer,
    widgets::{BuildCtx, Column},
};

fn main() -> Result<()> {
    luna::init_logging();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Retained demo")
            .with_inner_size(winit::dpi::LogicalSize::new(640, 480))
            .build(&event_loop)?,
    );

    let cloned_window = window.clone();
    let mut renderer =
        pollster::block_on(Renderer::new(&cloned_window, window.scale_factor() as f32))?;

    let (dirty_tx, dirty_rx) = std::sync::mpsc::channel();
    signals::init_reactivity(dirty_tx);

    let (count, set_count) = create_signal(0);

    let label = {
        let count = count.clone();
        move || format!("Click me: {}", count.get())
    };
    let (read_label, write_label) = create_signal(label());

    let on_click_action = move || {
        set_count.update(|c| *c += 1);
        write_label.set(label());
        log::info!("Clicked! New count: {}", count.get());
    };

    let button = Button::new(read_label).on_click(on_click_action);

    let root_widget = Column {
        spacing: 8.0,
        children: vec![Box::new(button)],
    };

    let mut root = Node::new(
        Box::new(root_widget),
        Rect {
            origin: vec2(50.0, 50.0),
            size: vec2(160.0, 80.0),
        },
        &mut BuildCtx,
    );

    for dirty_node_id in dirty_rx.try_iter() {
        root.mark_dirty_by_id(dirty_node_id);
    }

    let mut win_width = window.inner_size().width as f32;
    let mut focus_mgr = FocusManager::default();

    let _ = event_loop.run(move |event, elwt| match &event {
        Event::WindowEvent {
            window_id,
            event: WindowEvent::RedrawRequested,
        } if *window_id == window.id() => {
            renderer.begin_frame();
            root.layout(win_width);
            root.collect(&mut renderer);

            if let Err(e) = renderer.end_frame() {
                log::error!("frame: {e}");
            }
        }

        Event::WindowEvent { window_id, event } if *window_id == window.id() => {
            match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::Resized(sz) => {
                    win_width = sz.width as f32;
                    renderer.resize(*sz);
                    root.mark_dirty();
                }
                _ => {}
            }
            root.route_window_event(event, &mut focus_mgr, window.scale_factor());
        }

        Event::AboutToWait => window.request_redraw(),

        _ => {}
    });

    Ok(())
}
