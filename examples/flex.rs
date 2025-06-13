use std::sync::Arc;

use glam::{Vec2, vec2};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use luna::{
    layout::{Rect, node::Node},
    renderer::Renderer,
    style::{Align, Display, FlexDir, Justify, Style},
    widgets::{BuildCtx, Button, Widget},
    windowing::events::FocusManager,
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

    fn measure(&self, _max: f32) -> Vec2 {
        Vec2::ZERO
    }

    fn paint(
        &mut self,
        children: &mut [luna::layout::node::Node],
        layout: Rect,
        ren: &mut Renderer,
    ) {
        for child in children {
            if child.layout_rect.intersects(&layout) {
                child.collect(ren);
            }
        }
    }

    fn style(&self) -> Style {
        self.style
    }
}

fn main() -> luna::Result<()> {
    luna::init_logging();

    let event_loop = EventLoop::new()?;

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Flex demo")
            .with_inner_size(winit::dpi::LogicalSize::new(640, 120))
            .build(&event_loop)?,
    );

    let cloned_window = window.clone();
    let mut renderer =
        pollster::block_on(Renderer::new(&cloned_window, window.scale_factor() as f32))?;

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

    let mut root = Node::new(
        Box::new(row),
        Rect::new(vec2(0.0, 0.0), vec2(640.0, 120.0)),
        &mut BuildCtx,
    );

    let mut win_width = window.inner_size().width as f32;
    let mut focus_manager = FocusManager::default();

    window.request_redraw();

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

        Event::WindowEvent {
            window_id,
            event: w_event,
        } if *window_id == window.id() => {
            match w_event {
                WindowEvent::CloseRequested => elwt.exit(),

                WindowEvent::Resized(sz) => {
                    win_width = sz.width as f32;
                    renderer.resize(*sz);

                    root.set_rect(Rect::new(
                        vec2(0.0, 0.0),
                        vec2(sz.width as f32, sz.height as f32),
                    ));

                    root.mark_dirty();
                }
                _ => {}
            }
            root.route_window_event(w_event, &mut focus_manager, window.scale_factor());

            window.request_redraw();
        }
        _ => {}
    });

    Ok(())
}
