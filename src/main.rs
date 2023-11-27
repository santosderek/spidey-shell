mod openai;
mod user_interface;

use dotenv::dotenv;

use std::{error::Error, path::Path};

fn main() -> Result<(), Box<dyn Error>> {

    if !Path::new(".env").exists() {
        return Err("No .env file found. Please create one with your OpenAI API key.".into());
    }

    dotenv().ok();

    let mut main_window = user_interface::create_main_window();

    main_window.run();
    Ok(())
}
