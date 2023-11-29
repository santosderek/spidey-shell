extern crate spidey_shell;

use spidey_shell::user_interface;

use dotenv::dotenv;

use std::{error::Error, path::Path};

use dirs::home_dir;

fn main() -> Result<(), Box<dyn Error>> {
    let home_directory = home_dir().unwrap();
    let home_directory_env = home_directory.join(".env");

    if Path::new(".env").exists() {
        dotenv().ok();
    } else if Path::new(&home_directory_env).exists() {
        dotenv::from_filename(home_directory_env).ok();
    } else {
        return Err(
            "No .env file found in CWD or HOME. Please create one with your OpenAI API key.".into(),
        );
    }

    dotenv().ok();

    let mut main_window = user_interface::create_main_window();

    main_window.run();
    Ok(())
}
