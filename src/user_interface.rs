use cursive::CursiveRunnable;

use crate::menu::create_main_menu;

/// Create the main window for the application
pub fn create_main_window() -> CursiveRunnable {
    let mut main_window = cursive::default();
    main_window
        .load_toml(include_str!("./assets/theme.toml"))
        .unwrap();

    main_window.add_layer(create_main_menu());

    return main_window;
}
