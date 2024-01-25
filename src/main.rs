extern crate spidey_shell;

use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use dirs::home_dir;
use dotenv::dotenv;
use ratatui::{backend::CrosstermBackend, Terminal};
use spidey_shell::elm::run_event_loop;
use std::{error::Error, io::stdout, path::Path};

fn load_environemnt() -> Result<(), Box<dyn Error>> {
    let home_directory = home_dir().unwrap();
    let home_directory_env = home_directory.join(".env");

    if Path::new(".env").exists() {
        dotenv().ok();
    } else if Path::new(&home_directory_env).exists() {
        dotenv::from_filename(home_directory_env).ok();
    } else {
        return Err(
            "No .env file found in CWD or HOME. Please create one with your OpenAI API key [OPENAI_API_KEY].".into(),
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    match load_environemnt() {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    }

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let _ = run_event_loop(&mut terminal).await;

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
