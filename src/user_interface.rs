use cursive::{
    event::{Event, Key},
    CursiveRunnable,
};

use crate::openai::{create_chat_layout, submit_input};

/// Create the main window for the application
pub fn create_main_window() -> CursiveRunnable {
    let mut main_window = cursive::default();
    main_window
        .load_toml(include_str!("./assets/theme.toml"))
        .unwrap();

    main_window.add_layer(create_chat_layout());
    main_window.add_global_callback(Event::Ctrl(Key::Enter), submit_input);

    return main_window;
}
