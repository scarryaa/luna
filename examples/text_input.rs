use luna::{
    App, Element, Result, Text, TextInput,
    style::{Align, Display, FlexDir, Justify},
};

fn main() -> Result<()> {
    let app_ui = Element::new()
        .display(Display::Flex)
        .flex_direction(FlexDir::Column)
        .justify_content(Justify::Center)
        .align_items(Align::Stretch)
        .padding(16.0)
        .gap(16.0)
        .child(Text::new("Enter your name:"))
        .child(TextInput::new("e.g. Jane Doe"))
        .child(TextInput::new("Another input..."));

    App::new(app_ui)
        .with_title("Text Input Demo")
        .with_size(320, 240)
        .run()
}
