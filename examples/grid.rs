use luna::{
    App, Element, Result, Theme,
    style::Display,
    widgets::{Button, Widget},
};

fn main() -> Result<()> {
    let buttons: Vec<Box<dyn Widget>> = (1..=9)
        .map(|i| Box::new(Button::label(&format!("Button {i}"))) as Box<dyn Widget>)
        .collect();

    let theme = Theme::default();

    let ui = Element::new()
        .display(Display::Grid)
        .background_color(theme.color.surface)
        .padding(12.0)
        .gap(8.0)
        .grid_cols(3)
        .grid_row_height(32.0)
        .children(buttons);

    App::new(ui)
        .with_title("Grid Demo")
        .with_size(480, 200)
        .run()
}
