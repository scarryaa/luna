use glam::{Vec2, vec2};
use luna::layout::node::Node;
use luna::windowing::events::FocusManager;
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use luna::{
    Result,
    layout::Rect,
    renderer::Renderer,
    style::{Align, Display, FlexDir, Justify, Style},
    widgets::{BuildCtx, Column, Text, TextInput, Widget},
};

fn main() -> Result<()> {
    luna::init_logging();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Text Input demo")
            .with_inner_size(winit::dpi::LogicalSize::new(320, 240))
            .build(&event_loop)?,
    );

    let cloned_window = window.clone();

    let mut scale_factor = window.scale_factor();
    let mut logical_size = window.inner_size().to_logical::<f32>(scale_factor);

    let mut renderer = pollster::block_on(Renderer::new(&cloned_window, scale_factor as f32))?;

    let root_widget = Column {
        spacing: 16.0,
        children: vec![
            Box::new(Text {
                content: "Enter your name:".to_string(),
                ..Default::default()
            }),
            Box::new(TextInput::new("e.g. Jane Doe")),
            Box::new(TextInput::new("Another input...")),
        ],
    };

    let mut root_style = Style::default();
    root_style.display = Display::Flex;
    root_style.flex.dir = FlexDir::Column;
    root_style.flex.align = Align::Center;
    root_style.flex.justify = Justify::Center;
    root_style.padding = vec2(16.0, 16.0);

    #[derive(Clone)]
    struct Container {
        style: Style,
        child: Box<dyn Widget>,
    }
    impl Widget for Container {
        fn build(&self, _ctx: &mut BuildCtx) -> Vec<Box<dyn Widget>> {
            vec![self.child.clone()]
        }
        fn measure(&self, max: f32) -> Vec2 {
            self.child.measure(max)
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

    let container = Container {
        style: root_style,
        child: Box::new(root_widget),
    };

    let mut root = Node::new(
        Box::new(container),
        Rect {
            origin: vec2(0.0, 0.0),
            size: vec2(logical_size.width, logical_size.height),
        },
        &mut BuildCtx,
    );

    let mut focus_mgr = FocusManager::default();

    let _ = event_loop.run(move |event, elwt| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            root.route_window_event(event, &mut focus_mgr, scale_factor);

            match event {
                WindowEvent::CloseRequested => elwt.exit(),

                WindowEvent::Resized(physical_size) => {
                    log::info!("Event: Resized to {:?}", physical_size);
                    logical_size = physical_size.to_logical(scale_factor);
                    renderer.resize(*physical_size);
                    root.set_rect(Rect::new(
                        Vec2::ZERO,
                        vec2(logical_size.width as f32, logical_size.height as f32),
                    ));
                    root.mark_dirty();
                }

                WindowEvent::ScaleFactorChanged { .. } => {
                    scale_factor = window.scale_factor();
                    renderer.set_scale_factor(scale_factor as f32);
                    log::info!("Event: ScaleFactorChanged to {}", scale_factor);
                }

                WindowEvent::RedrawRequested => {
                    if logical_size.width <= 0.0 || logical_size.height <= 0.0 {
                        return;
                    }

                    root.layout(logical_size.width as f32);

                    renderer.begin_frame();
                    root.collect(&mut renderer);

                    if let Err(e) = renderer.end_frame() {
                        log::error!("frame error: {e}");
                    }
                }
                _ => {}
            }
        }
        Event::AboutToWait => {
            window.request_redraw();
        }
        _ => {}
    });

    Ok(())
}
