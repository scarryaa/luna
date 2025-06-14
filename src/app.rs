use crate::{
    Result,
    layout::{Rect, node::Node},
    renderer::Renderer,
    signals,
    widgets::{BuildCtx, Widget},
    windowing::events::FocusManager,
};
use glam::{Vec2, vec2};
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

struct WindowConfig {
    title: String,
    size: winit::dpi::LogicalSize<u32>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Luna App".to_string(),
            size: winit::dpi::LogicalSize::new(800, 600),
        }
    }
}

pub struct App {
    root_widget: Box<dyn Widget>,
    window_config: WindowConfig,
}

impl App {
    pub fn new(root_widget: impl Widget + 'static) -> Self {
        Self {
            root_widget: Box::new(root_widget),
            window_config: WindowConfig::default(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.window_config.title = title.into();
        self
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.window_config.size = winit::dpi::LogicalSize::new(width, height);
        self
    }

    pub fn run(self) -> Result<()> {
        crate::init_logging();
        log::info!("Starting {}...", &self.window_config.title);

        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);

        let window = Arc::new(
            WindowBuilder::new()
                .with_title(&self.window_config.title)
                .with_inner_size(self.window_config.size)
                .build(&event_loop)?,
        );

        let window_clone = window.clone();
        let mut renderer =
            pollster::block_on(Renderer::new(&window_clone, window.scale_factor() as f32))?;

        let (dirty_tx, dirty_rx) = std::sync::mpsc::channel();
        signals::init_reactivity(dirty_tx);

        let initial_size = window.inner_size();
        let mut root = Node::new(
            self.root_widget,
            Rect::new(
                Vec2::ZERO,
                vec2(initial_size.width as f32, initial_size.height as f32),
            ),
            &mut BuildCtx,
        );

        let mut win_width = initial_size.width as f32;
        let mut focus_mgr = FocusManager::default();

        let _ = event_loop.run(move |event, elwt| {
            for dirty_node_id in dirty_rx.try_iter() {
                root.mark_dirty_by_id(dirty_node_id);
            }

            match &event {
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::RedrawRequested,
                } if *window_id == window.id() => {
                    renderer.begin_frame();
                    root.layout(win_width);
                    root.collect(&mut renderer);

                    if let Err(e) = renderer.end_frame() {
                        log::error!("frame error: {e}");
                    }
                }

                Event::WindowEvent { window_id, event } if *window_id == window.id() => {
                    root.route_window_event(event, &mut focus_mgr, window.scale_factor());

                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(sz) => {
                            win_width = sz.width as f32;
                            renderer.resize(*sz);
                            root.set_rect(Rect::new(
                                Vec2::ZERO,
                                vec2(sz.width as f32, sz.height as f32),
                            ));
                            root.mark_dirty();
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
}
