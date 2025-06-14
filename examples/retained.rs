use luna::{
    App, Button, Element, Result, Text,
    signals::{create_memo, create_signal},
    style::{Align, Display, FlexDir, Justify, tokens::Colour},
};

fn main() -> Result<()> {
    let (count, set_count) = create_signal(0);

    let label_memo = create_memo({
        let count = count.clone();
        move || format!("Click me: {}", count.get())
    });

    let heading_memo = create_memo({
        let count = count.clone();
        move || format!("Current count is: {}", count.get())
    });

    let on_click_action = move || {
        set_count.update(|c| *c += 1);
        log::info!("Clicked! New count: {}", count.get());
    };

    let app_ui = Element::new()
        .display(Display::Flex)
        .flex_direction(FlexDir::Column)
        .justify_content(Justify::Center)
        .align_items(Align::Center)
        .gap(16.0)
        .background_color(Colour::SURFACE)
        .child(Text {
            content: heading_memo,
            color: Colour::TEXT.into(),
            size: 24.0,
        })
        .child(Button::new(label_memo).on_click(on_click_action));

    App::new(app_ui)
        .with_title("Retained Demo")
        .with_size(640, 480)
        .run()
}
