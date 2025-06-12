use luna::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<()> {
    luna::init_logging();
    log::info!("Starting Luna immediate mode example");

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = WindowBuilder::new()
        .with_title("Immediate")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)?;

    let mut renderer = pollster::block_on(luna::Renderer::new(&window))?;

    // Main event loop
    let _ = event_loop.run(|event, elwt| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => {
                        log::info!("Close requested, exiting");
                        elwt.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        log::info!("Window resized to {:?}", physical_size);
                        renderer.resize(*physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        renderer.begin_frame();

                        renderer.draw_rect(
                            Vec2::new(100.0, 100.0),       // position
                            Vec2::new(200.0, 150.0),       // size
                            Vec4::new(0.3, 0.6, 0.9, 1.0), // color
                        );

                        renderer.draw_rect(
                            Vec2::new(350.0, 200.0),
                            Vec2::new(150.0, 100.0),
                            Vec4::new(0.9, 0.3, 0.3, 1.0),
                        );

                        renderer.draw_text(
                            "Hello Luna!",
                            Vec2::new(50.0, 50.0),
                            Vec4::new(1.0, 1.0, 1.0, 1.0),
                            24.0,
                        );

                        if let Err(e) = renderer.end_frame() {
                            log::error!("Failed to end frame: {}", e);
                        }
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    });

    Ok(())
}
