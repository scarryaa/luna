use glam::{Vec2, vec2};
use luna::{
    layout::Rect,
    renderer::Renderer,
    style::{Align, Display, FlexDir, Justify, Style},
    widgets::{BuildCtx, Button, Widget},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[derive(Clone)]
struct FlexRow {
    style: Style,
    children: Vec<Box<dyn Widget>>,
}

impl Widget for FlexRow {
    fn build(&self, _ctx: &mut BuildCtx) -> Vec<Box<dyn Widget>> {
        self.children.clone()
    }

    fn measure(&self, _max_w: f32) -> Vec2 {
        Vec2::ZERO
    } // flex sizes children

    fn paint(&self, _rect: Rect, _ren: &mut Renderer) {}

    fn style(&self) -> Style {
        self.style
    }
}

fn main() -> luna::Result<()> {
    luna::init_logging();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = WindowBuilder::new()
        .with_title("Flex demo")
        .with_inner_size(winit::dpi::LogicalSize::new(640, 120))
        .build(&event_loop)?;

    let mut renderer = pollster::block_on(Renderer::new(&window))?;

    // three buttons that stretch & space-between
    let mut ctx = BuildCtx;
    let btn = |txt| Box::new(Button::label(txt)) as Box<dyn Widget>;
    let row = FlexRow {
        style: Style {
            display: Display::Flex,
            flex: luna::style::Flex {
                dir: FlexDir::Row,
                justify: Justify::SpaceBetween,
                align: Align::Center,
            },
            padding: vec2(12.0, 12.0),
            ..Default::default()
        },
        children: vec![btn("One"), btn("Two"), btn("Three")],
    };

    let mut root = luna::layout::node::Node::new(
        Box::new(row),
        Rect::new(vec2(0.0, 0.0), vec2(640.0, 120.0)),
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
