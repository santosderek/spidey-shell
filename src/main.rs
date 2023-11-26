use cursive::views::{Dialog, TextView};

fn main() {
    let mut main_window = cursive::default();

    let vertical_chat = LinearLayout::::

    main_window.add_layer(
        Dialog::around(TextView::new("Hello Dialog!"))
            .title("Spidey Shell")
            .button("Quit", |_window| _window.quit()),
    );

    main_window.run();
}
