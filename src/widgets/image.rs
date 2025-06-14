use crate::{
    Widget,
    layout::{Rect, node::Node},
    renderer::Renderer,
    style::{Style, Theme},
};
use glam::Vec2;

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
    dimensions: Option<(u32, u32)>,
}

impl Image {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            fit: ImageFit::default(),
            dimensions: None,
        }
    }

    pub fn fit(mut self, fit: ImageFit) -> Self {
        self.fit = fit;
        self
    }

    fn get_dimensions(&mut self) -> Option<(u32, u32)> {
        if self.dimensions.is_none() {
            self.dimensions = image::image_dimensions(&self.path).ok();
        }
        self.dimensions
    }
}

impl Widget for Image {
    fn style(&self) -> Style {
        Style {
            flex_grow: 1.0,
            ..Default::default()
        }
    }

    fn measure(&self, _max_width: f32, _theme: &Theme) -> Vec2 {
        Vec2::ZERO
    }

    fn paint(&mut self, node: &mut Node, ren: &mut Renderer, _theme: &Theme) {
        let container_rect = node.layout_rect;

        if container_rect.size.x <= 0.0 || container_rect.size.y <= 0.0 {
            return;
        }

        let mut draw_rect = container_rect;

        if self.fit != ImageFit::Fill {
            if let Some((img_w, img_h)) = self.get_dimensions() {
                let img_w = img_w as f32;
                let img_h = img_h as f32;
                let container_w = container_rect.size.x;
                let container_h = container_rect.size.y;

                let img_aspect = img_w / img_h;
                let container_aspect = container_w / container_h;

                let (new_w, new_h) = match self.fit {
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
                    ImageFit::Fill => (container_w, container_h),
                };

                let new_x = container_rect.origin.x + (container_w - new_w) / 2.0;
                let new_y = container_rect.origin.y + (container_h - new_h) / 2.0;

                draw_rect = Rect::new(Vec2::new(new_x, new_y), Vec2::new(new_w, new_h));
            }
        }

        ren.draw_image(&self.path, draw_rect);
    }
}
