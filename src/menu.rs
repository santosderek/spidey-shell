use cursive::{views::{Button, Dialog, DummyView, LinearLayout, TextView, NamedView}, view::Nameable};

/// Creates a list of all selectable main menu items
pub fn create_main_menu() -> NamedView<Dialog> {
    let mut main_menu = LinearLayout::vertical();

    main_menu.add_child(TextView::new("Welcome to Spidey Shell!"));
    main_menu.add_child(DummyView);
    main_menu.add_child(
        LinearLayout::vertical().child(Button::new("Chat", |window| {
            let layout = crate::openai::create_chat_layout();
            window.pop_layer();
            window.add_layer(layout);
        })),
    );
    main_menu.add_child(DummyView);
    main_menu
        .add_child(LinearLayout::horizontal().child(Button::new("Quit", |window| window.quit())));

    return Dialog::around(main_menu).title("Spidey Shell").with_name("main_menu");
}
