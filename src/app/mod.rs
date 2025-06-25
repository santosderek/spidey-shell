// Main application module that ties everything together
pub mod events;
pub mod input;
pub mod models;
pub mod state;
pub mod ui;

// Re-export the main components
pub use events::handle_key_event;
pub use state::AppState;
pub use ui::render;
