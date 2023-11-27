mod user_interface;
mod ai;

use dotenv::dotenv;

fn main() {

    dotenv().ok();

    let mut main_window = user_interface::create_main_window();

    main_window.run();
}
