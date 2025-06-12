use glam::vec2;
use luna::layout::node::Node;
use std::cell::RefCell;
use std::rc::Rc;
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

    let window = WindowBuilder::new()
        .with_title("Retained demo")
        .with_inner_size(winit::dpi::LogicalSize::new(640, 480))
        .build(&event_loop)?;

    let mut renderer = pollster::block_on(Renderer::new(&window))?;

    // Build the UI tree
    let mut ctx = BuildCtx;
    let button = Button {
        label: "Click".into(),
        on_click: Rc::new(RefCell::new(|| log::info!("clicked!"))),
        hovered: false,
    };

    let root_widget = Column {
        spacing: 8.0,
        children: vec![Box::new(button)],
    };

    let mut root = Node::new(
        Box::new(root_widget),
        Rect {
            origin: vec2(50.0, 50.0),
            size: vec2(140.0, 80.0),
        },
        &mut ctx,
    );

    let mut win_width = window.inner_size().width as f32;

    // Event / Render loop
    let _ = event_loop.run(|event, elwt| match event {
        Event::WindowEvent {
            window_id,
            event: WindowEvent::RedrawRequested,
        } if window_id == window.id() => {
            renderer.begin_frame();

            root.layout(win_width);
            root.collect(&mut renderer);

            if let Err(e) = renderer.end_frame() {
                log::error!("frame: {e}");
            }
        }
        Event::WindowEvent {
            window_id,
            ref event,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::Resized(sz) => {
                win_width = sz.width as f32;
                renderer.resize(*sz);
            }
            WindowEvent::MouseInput { .. }
            | WindowEvent::CursorMoved { .. }
            | WindowEvent::CursorLeft { .. } => {
                if let WindowEvent::CursorMoved { position, .. } = event {
                    let pos = vec2(position.x as f32, position.y as f32);
                    root.hit(pos, event);
                } else {
                    root.hit(vec2(0.0, 0.0), event);
                }
            }
            _ => {}
        },
        Event::AboutToWait => window.request_redraw(),
        _ => {}
    });
    Ok(())
}
