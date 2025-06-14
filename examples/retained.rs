use luna::{
    Align, App, Button, Display, Element, FlexDir, Justify, Result, Text, Theme,
    signals::{create_memo, create_signal},
};

fn main() -> Result<()> {
    let theme = Theme::default();
    let (count, set_count) = create_signal(0);

    let heading_memo = create_memo({
        let count = count.clone();
        move || format!("Current count is: {}", count.get())
    });

    let label_memo = create_memo({
        let count = count.clone();
        move || format!("Click me: {}", count.get())
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
        .background_color(theme.color.surface)
        .child(Text::new(heading_memo).with_size(24.0))
        .child(Button::new(label_memo).on_click(on_click_action));

    App::new(app_ui)
        .with_title("Retained Demo")
        .with_size(640, 480)
        .run()
}
