use luna::{
    Align, App, Display, Element, FlexDir, Image, Justify, Result, widgets::image::ImageFit,
};

fn main() -> Result<()> {
    let ui = Element::new()
        .display(Display::Flex)
        .flex_direction(FlexDir::Column)
        .justify_content(Justify::Center)
        .align_items(Align::Center)
        .gap(16.0)
        .child(
            Element::new()
                .display(Display::Flex)
                .width(128.0)
                .height(256.0)
                .child(Image::new("assets/ferris.png").fit(ImageFit::Contain)),
        )
        .child(luna::widgets::Text::new("Image Widget Demo"));

    App::new(ui)
        .with_title("Image Demo")
        .with_size(640, 480)
        .run()
}
