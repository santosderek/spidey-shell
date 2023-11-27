mod user_interface;
mod ai;

use dotenv::dotenv;

use std::path::Path;
use std::fs::File;

fn main() {

    // Check if file exists
    
    if !Path::new(".env").exists() {
        File::create(".env").expect("Failed to create .env file");
    }

    dotenv().ok();

    let mut main_window = user_interface::create_main_window();

    main_window.run();
}
