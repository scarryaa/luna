use luna::{
    App, Element, Result,
    style::tokens::Colour,
    style::{Align, Display, FlexDir, Justify},
    widgets::Button,
};

fn main() -> Result<()> {
    let ui = Element::new()
        .display(Display::Flex)
        .flex_direction(FlexDir::Row)
        .justify_content(Justify::SpaceBetween)
        .align_items(Align::Center)
        .padding(12.0)
        .background_color(Colour::SURFACE)
        .child(Button::label("One"))
        .child(Button::label("Two"))
        .child(Button::label("Three"));

    App::new(ui)
        .with_title("Flex Demo")
        .with_size(640, 120)
        .run()
}
