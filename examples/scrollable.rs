use luna::{
    App, Element, Result,
    style::{Display, FlexDir},
    widgets::{Button, Scrollable, Widget},
};

fn main() -> Result<()> {
    let many_buttons: Vec<Box<dyn Widget>> = (0..100)
        .map(|i| Box::new(Button::label(&format!("Button #{i}"))) as Box<dyn Widget>)
        .collect();

    let content = Element::new()
        .display(Display::Flex)
        .flex_direction(FlexDir::Column)
        .gap(8.0)
        .padding(12.0)
        .children(many_buttons);

    let scrollable_ui = Scrollable::new(content);

    App::new(scrollable_ui)
        .with_title("Scrollable Demo")
        .with_size(240, 480)
        .run()
}
