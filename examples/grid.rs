use glam::{Vec2, vec2};
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use luna::{
    layout::{Rect, node::Node},
    renderer::Renderer,
    style::{Display, Grid as GridStyle, Style},
    widgets::{BuildCtx, Button, Widget},
    windowing::events::FocusManager,
};

#[derive(Clone)]
struct Grid {
    style: Style,
    children: Vec<Box<dyn Widget>>,
}

impl Grid {
    fn new(cols: u16, children: Vec<Box<dyn Widget>>) -> Self {
        Self {
            children,
            style: Style {
                display: Display::Grid,
                grid: GridStyle {
                    cols,
                    row_height: 32.0,
                    gap: vec2(8.0, 8.0),
                },
                padding: vec2(12.0, 12.0),
                ..Default::default()
            },
        }
    }
}

impl Widget for Grid {
    fn build(&self, _ctx: &mut BuildCtx) -> Vec<Box<dyn Widget>> {
        self.children.clone()
    }

    fn measure(&self, _max_width: f32) -> Vec2 {
        Vec2::ZERO
    }

    fn paint(
        &mut self,
        children: &mut [luna::layout::node::Node],
        _rect: Rect,
        _ren: &mut Renderer,
    ) {
        for child in children {
            child.collect(_ren);
        }
    }

    fn style(&self) -> Style {
        self.style
    }
}

fn main() -> luna::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Grid demo")
            .with_inner_size(winit::dpi::LogicalSize::new(480, 320))
            .build(&event_loop)?,
    );
    window.request_redraw();

    let cloned_window = window.clone();
    let mut renderer = pollster::block_on(Renderer::new(&cloned_window))?;
    let make_btn = |i| Box::new(Button::label(&format!("Btn {i}"))) as Box<dyn Widget>;
    let grid = Grid::new(3, (1..10).map(make_btn).collect());

    let mut root = Node::new(
        Box::new(grid),
        Rect::new(vec2(0.0, 0.0), vec2(480.0, 320.0)),
        &mut BuildCtx,
    );

    let mut win_width = window.inner_size().width as f32;
    let mut focus_manager = FocusManager::default();

    let _ = event_loop.run(move |event, elwt| match &event {
        Event::WindowEvent {
            window_id,
            event: WindowEvent::RedrawRequested,
        } if *window_id == window.id() => {
            renderer.begin_frame();
            root.layout(win_width);
            root.collect(&mut renderer);
            renderer.end_frame().ok();
        }
        Event::WindowEvent { window_id, event } if *window_id == window.id() => {
            match event {
                WindowEvent::CloseRequested => elwt.exit(),

                WindowEvent::Resized(sz) => {
                    win_width = sz.width as f32;

                    renderer.resize(*sz);

                    root.set_rect(Rect::new(
                        vec2(0.0, 0.0),
                        vec2(sz.width as f32, sz.height as f32),
                    ));

                    root.mark_dirty();

                    window.request_redraw();
                }

                _ => {}
            }

            root.route_window_event(event, &mut focus_manager);
            window.request_redraw();
        }
        _ => {}
    });

    Ok(())
}
