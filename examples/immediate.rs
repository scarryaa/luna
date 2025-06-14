use luna::{App, Canvas, Result, Vec2, Vec4};

fn main() -> Result<()> {
    let drawing_logic = |renderer: &mut luna::Renderer| {
        renderer.draw_rect(
            Vec2::new(100.0, 100.0),       // position
            Vec2::new(200.0, 150.0),       // size
            Vec4::new(0.3, 0.6, 0.9, 1.0), // color
        );

        renderer.draw_rect(
            Vec2::new(350.0, 200.0),
            Vec2::new(150.0, 100.0),
            Vec4::new(0.9, 0.3, 0.3, 1.0),
        );

        renderer.draw_text(
            "Hello Luna!",
            Vec2::new(50.0, 50.0),
            Vec4::new(1.0, 1.0, 1.0, 1.0),
            24.0,
        );
    };

    let ui = Canvas::new(drawing_logic);

    App::new(ui)
        .with_title("Immediate Demo")
        .with_size(800, 600)
        .run()
}
