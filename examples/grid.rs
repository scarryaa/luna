use glam::{Vec2, vec2};
use luna::{
    layout::Rect,
    renderer::Renderer,
    style::{Display, Grid as GridStyle, Style},
    widgets::{BuildCtx, Button, Widget},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
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

    fn measure(&self, _max: f32) -> Vec2 {
        Vec2::ZERO
    } // grid sizes children

    fn paint(&self, _rect: Rect, _ren: &mut Renderer) {}

    fn style(&self) -> Style {
        self.style
    }
}

fn main() -> luna::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = WindowBuilder::new()
        .with_title("Grid demo")
        .with_inner_size(winit::dpi::LogicalSize::new(480, 320))
        .build(&event_loop)?;

    let mut renderer = pollster::block_on(Renderer::new(&window))?;

    let mut ctx = BuildCtx;
    let make = |i| Box::new(Button::label(&format!("Btn {i}"))) as Box<dyn Widget>;
    let grid = Grid::new(3, (1..10).map(make).collect());

    let mut root = luna::layout::node::Node::new(
        Box::new(grid),
        Rect::new(vec2(0.0, 0.0), vec2(480.0, 320.0)),
        &mut ctx,
    );

    let mut win_width = window.inner_size().width as f32;

    let _ = event_loop.run(|event, elwt| match event {
        Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::Resized(sz) => {
                win_width = sz.width as f32;
                renderer.resize(sz);
            }
            WindowEvent::RedrawRequested => {
                renderer.begin_frame();
                root.layout(win_width);
                root.collect(&mut renderer);
                renderer.end_frame().ok();
            }
            _ => {}
        },
        Event::AboutToWait => window.request_redraw(),
        _ => {}
    });
    Ok(())
}
