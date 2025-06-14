use luna::{
    App, Button, Element, Result, Text,
    signals::create_signal,
    style::{Align, Display, FlexDir, Justify, tokens::Colour},
};

fn main() -> Result<()> {
    let (count, set_count) = create_signal(0);

    let label_signal = {
        let count = count.clone();
        create_signal(format!("Click me: {}", count.get()))
    };

    let heading_signal = {
        let count = count.clone();
        create_signal(format!("Current count is: {}", count.get()))
    };

    let on_click_action = move || {
        set_count.update(|c| *c += 1);

        label_signal.1.set(format!("Click me: {}", count.get()));
        heading_signal
            .1
            .set(format!("Current count is: {}", count.get()));

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
            content: heading_signal.0,
            color: Colour::TEXT.into(),
            size: 24.0,
        })
        .child(Button::new(label_signal.0).on_click(on_click_action));

    App::new(app_ui)
        .with_title("Retained Demo")
        .with_size(640, 480)
        .run()
}
