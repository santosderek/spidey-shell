use cursive::{
    view::Nameable,
    views::{Button, Dialog, DummyView, LinearLayout, NamedView, TextView},
};

use crate::openai::fill_window_with_history;

/// Creates a list of all selectable main menu items
pub fn create_main_menu() -> NamedView<Dialog> {
    let mut main_menu = LinearLayout::vertical();

    main_menu.add_child(TextView::new("Welcome to Spidey Shell!"));
    main_menu.add_child(DummyView);
    main_menu.add_child(
        LinearLayout::vertical().child(Button::new("Chat", |window| {
            window.pop_layer();
            window.add_layer(crate::openai::create_chat_layout());
            fill_window_with_history(window);
        })),
    );
    main_menu.add_child(DummyView);
    main_menu
        .add_child(LinearLayout::horizontal().child(Button::new("Quit", |window| window.quit())));

    return Dialog::around(main_menu)
        .title("Spidey Shell")
        .with_name("main_menu");
}
