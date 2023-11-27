mod user_interface;
mod ai;

fn main() {
    let mut main_window = user_interface::create_main_window();

    main_window.run();
}
