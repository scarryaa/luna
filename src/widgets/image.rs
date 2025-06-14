use std::cell::RefCell;

use crate::{
    Widget,
    layout::{Rect, node::Node},
    renderer::Renderer,
    style::{Style, Theme},
};
use glam::{Vec2, vec2};

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum ImageFit {
    #[default]
    Fill,
    Contain,
    Cover,
}

#[derive(Clone)]
pub struct Image {
    path: String,
    fit: ImageFit,
    dimensions: RefCell<Option<(u32, u32)>>,
    style: Style,
}

impl Image {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            fit: ImageFit::default(),
            dimensions: RefCell::new(None),
            style: Style {
                flex_grow: 0.0,
                ..Default::default()
            },
        }
    }

    pub fn fit(mut self, fit: ImageFit) -> Self {
        self.fit = fit;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.style.width = Some(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.style.height = Some(height);
        self
    }

    fn get_dimensions(&self) -> Option<(u32, u32)> {
        if self.dimensions.borrow().is_none() {
            let new_dims = image::image_dimensions(&self.path).ok();
            *self.dimensions.borrow_mut() = new_dims;
        }
        *self.dimensions.borrow()
    }
}

impl Widget for Image {
    fn style(&self) -> Style {
        self.style
    }

    fn measure(
        &self,
        _max_width: f32,
        _theme: &Theme,
        _font_system: &mut cosmic_text::FontSystem,
    ) -> Vec2 {
        if let (Some(w), Some(h)) = (self.style.width, self.style.height) {
            return vec2(w, h);
        }

        if let Some((w, h)) = self.get_dimensions() {
            return vec2(w as f32, h as f32);
        }

        Vec2::ZERO
    }

    fn paint(&mut self, node: &mut Node, ren: &mut Renderer, _theme: &Theme) {
        let container_rect = node.layout_rect;

        if container_rect.size.x <= 0.0 || container_rect.size.y <= 0.0 {
            return;
        }

        let draw_rect = if let Some((img_w, img_h)) = self.get_dimensions() {
            let img_w = img_w as f32;
            let img_h = img_h as f32;
            let container_w = container_rect.size.x;
            let container_h = container_rect.size.y;

            let img_aspect = img_w / img_h;
            let container_aspect = container_w / container_h;

            let (new_w, new_h) = match self.fit {
                ImageFit::Fill => (container_w, container_h),
                ImageFit::Contain => {
                    if img_aspect > container_aspect {
                        (container_w, container_w / img_aspect)
                    } else {
                        (container_h * img_aspect, container_h)
                    }
                }
                ImageFit::Cover => {
                    if img_aspect > container_aspect {
                        (container_h * img_aspect, container_h)
                    } else {
                        (container_w, container_w / img_aspect)
                    }
                }
            };

            let new_x = container_rect.origin.x + (container_w - new_w) / 2.0;
            let new_y = container_rect.origin.y + (container_h - new_h) / 2.0;

            Rect::new(Vec2::new(new_x, new_y), Vec2::new(new_w, new_h))
        } else {
            container_rect
        };

        ren.draw_image(&self.path, draw_rect);
    }
}
