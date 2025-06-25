// Main application module that ties everything together
pub mod models;
pub mod state;
pub mod input;
pub mod events;
pub mod ui;

// Re-export the main components
pub use state::AppState;
pub use events::handle_key_event;
pub use ui::render;