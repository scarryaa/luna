use luna::{
    App, Checkbox, Element, Result, Text,
    signals::{create_memo, create_signal},
    style::{Align, Display, FlexDir, Justify},
};

fn main() -> Result<()> {
    let (is_checked, set_is_checked) = create_signal(false);

    let status_text = create_memo({
        let is_checked = is_checked.clone();
        move || {
            if is_checked.get() {
                "Status: Checked!".to_string()
            } else {
                "Status: Unchecked.".to_string()
            }
        }
    });

    let app_ui = Element::new()
        .display(Display::Flex)
        .flex_direction(FlexDir::Column)
        .justify_content(Justify::Center)
        .align_items(Align::Center)
        .gap(16.0)
        .child(Checkbox::new(
            "I agree to the terms".to_string(),
            (is_checked, set_is_checked),
        ))
        .child(Text::new(status_text));

    App::new(app_ui)
        .with_title("Checkbox Demo")
        .with_size(400, 300)
        .run()
}
