use std::error::Error;
use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use ratatui::backend::Backend;
use ratatui::Terminal;

use crate::mcp::MCPServerManager;
use crate::openai::AzureOpenAIClient;
use crate::persistence::config::AppConfig;
use crate::persistence::ConversationStore;

use super::input;
use super::models::InputMode;
use super::state::AppState;
use super::ui;

/// Process a keyboard event based on the current input mode
pub async fn handle_key_event(
    key_code: KeyCode,
    app: &mut AppState,
    config: &mut AppConfig,
    store: &ConversationStore,
    oaiclient: &AzureOpenAIClient,
    mcp_manager: &mut Option<MCPServerManager>,
) -> Result<bool, Box<dyn Error>> {
    // Handle keys differently based on the current input mode
    match app.input_mode {
        InputMode::Normal => {
            // In normal mode, handle vim-style navigation
            match key_code {
                KeyCode::Char('q') => return Ok(true), // Signal to quit
                KeyCode::Char('j') => app.scroll_down(),
                KeyCode::Char('k') => app.scroll_up(),
                KeyCode::Char('g') => app.scroll_to_top(), // gg in vim
                KeyCode::Char('G') => app.scroll_to_bottom(),
                KeyCode::Char('u') => app.scroll_half_page_up(10), // Simplified Ctrl+u
                KeyCode::Char('d') => app.scroll_half_page_down(10), // Simplified Ctrl+d
                KeyCode::Char('b') => app.scroll_page_up(20),      // Simplified Ctrl+b
                KeyCode::Char('f') => app.scroll_page_down(20),    // Simplified Ctrl+f
                KeyCode::Char('i') => app.enter_insert_mode(),
                KeyCode::Char('/') => app.start_forward_search(),
                KeyCode::Char('?') => app.start_backward_search(),
                KeyCode::Char('n') => app.search_in_chat(), // Repeat last search
                KeyCode::Up => input::handle_navigation_up(app),
                KeyCode::Down => input::handle_navigation_down(app),
                // Toggle MCP server with M key
                KeyCode::Char('M') => {
                    if app.current_conversation_id.is_some() && !app.mcp_servers.is_empty() {
                        if let Some(server_name) = app.mcp_servers.keys().next().cloned() {
                            input::toggle_mcp_server(app, store, &server_name)?;
                        }
                    }
                }
                _ => {}
            }
        }

        InputMode::Insert => {
            // In insert mode, handle text input and commands
            match key_code {
                KeyCode::Char('q') => {
                    if app.input.is_empty() {
                        return Ok(true); // Signal to quit if input is empty
                    } else {
                        input::handle_text_input(app, 'q');
                    }
                }
                KeyCode::Up => input::handle_navigation_up(app),
                KeyCode::Down => input::handle_navigation_down(app),
                KeyCode::Esc => app.enter_normal_mode(),
                KeyCode::Enter => {
                    let conversation = input::handle_conversation_creation(app, config, store)?;

                    // Process MCP server mentions if available
                    input::handle_mcp_server_mentions(app, store, mcp_manager)?;

                    input::handle_message_processing(app, &conversation, oaiclient, mcp_manager)
                        .await?;
                    input::handle_persistence(app, &conversation, store)?;

                    // After sending a message, scroll to the bottom
                    app.scroll_to_bottom();
                }
                KeyCode::Backspace => input::handle_backspace(app),
                KeyCode::Char(c) => input::handle_text_input(app, c),
                _ => {}
            }
        }

        InputMode::Command => {
            // Handle command mode input (like vim's command line)
            match key_code {
                KeyCode::Esc => app.enter_normal_mode(),
                KeyCode::Enter => {
                    // Process command - for now, just go back to normal mode
                    app.enter_normal_mode();
                }
                KeyCode::Backspace => {
                    if !app.search_query.is_empty() {
                        app.search_query.pop();
                    }
                }
                KeyCode::Char(c) => {
                    app.search_query.push(c);
                }
                _ => {}
            }
        }

        InputMode::Visual => {
            // In visual mode (for selecting text/messages)
            match key_code {
                KeyCode::Esc => app.enter_normal_mode(),
                KeyCode::Char('j') => app.scroll_down(),
                KeyCode::Char('k') => app.scroll_up(),
                KeyCode::Up => input::handle_navigation_up(app),
                KeyCode::Down => input::handle_navigation_down(app),
                _ => {}
            }
        }
    }

    // Handle search mode separately (can be active in any mode)
    if let Some(_mode) = app.search_mode {
        match key_code {
            KeyCode::Esc => {
                app.search_mode = None;
                app.search_query.clear();
            }
            KeyCode::Enter => {
                app.search_in_chat();
                app.search_mode = None; // Exit search mode after search
            }
            KeyCode::Backspace => {
                if !app.search_query.is_empty() {
                    app.search_query.pop();
                }
            }
            KeyCode::Char(c) => {
                app.search_query.push(c);
            }
            _ => {}
        }
    }

    Ok(false) // Continue running
}

/// Main application event loop
pub async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut AppState,
    config: &mut AppConfig,
    store: &ConversationStore,
    oaiclient: &AzureOpenAIClient,
    mcp_manager: &mut Option<MCPServerManager>,
) -> Result<(), Box<dyn Error>>
where
    B::Error: 'static + Error,
{
    loop {
        // Render the UI
        terminal.draw(|f| ui::render(f, app))?;

        // Poll for events
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                // Handle key event
                let should_quit =
                    handle_key_event(key.code, app, config, store, oaiclient, mcp_manager).await?;

                if should_quit {
                    return Ok(());
                }
            }
        }
    }
}

