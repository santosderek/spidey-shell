use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    spidey_shell::run::run_app().await
}

