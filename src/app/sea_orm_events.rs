use std::error::Error;
use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use ratatui::Terminal;
use ratatui::backend::Backend;

use crate::openai::AzureOpenAIClient;
use crate::persistence::seaorm_store::SeaOrmStore;
use crate::persistence::config::AppConfig;
use crate::mcp::MCPServerManager;

use super::sea_orm_state::SeaOrmAppState;
use super::models::{InputMode, SearchMode};
use super::sea_orm_input as input;
use super::ui;

/// Process a keyboard event based on the current input mode
pub async fn handle_key_event(
    key_code: KeyCode,
    app: &mut SeaOrmAppState,
    config: &mut AppConfig,
    store: &SeaOrmStore,
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
                KeyCode::Char('b') => app.scroll_page_up(20), // Simplified Ctrl+b
                KeyCode::Char('f') => app.scroll_page_down(20), // Simplified Ctrl+f
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
                            input::toggle_mcp_server(app, store, &server_name).await?;
                        }
                    }
                }
                _ => {}
            }
        },
        
        InputMode::Insert => {
            // In insert mode, handle text input and commands
            match key_code {
                KeyCode::Char('q') => {
                    if app.input.is_empty() {
                        return Ok(true); // Signal to quit if input is empty
                    } else {
                        input::handle_text_input(app, 'q');
                    }
                },
                KeyCode::Up => input::handle_navigation_up(app),
                KeyCode::Down => input::handle_navigation_down(app),
                KeyCode::Esc => app.enter_normal_mode(),
                KeyCode::Enter => {
                    let conversation = input::handle_conversation_creation(app, config, store).await?;
                    
                    // Process MCP server mentions if available
                    input::handle_mcp_server_mentions(app, store, mcp_manager).await?;
                    
                    input::handle_message_processing(app, &conversation, oaiclient, mcp_manager).await?;
                    input::handle_persistence(app, &conversation, store).await?;
                    
                    // After sending a message, scroll to the bottom
                    app.scroll_to_bottom();
                }
                KeyCode::Backspace => input::handle_backspace(app),
                KeyCode::Char(c) => input::handle_text_input(app, c),
                _ => {}
            }
        },
        
        InputMode::Command => {
            // Handle command mode input (like vim's command line)
            match key_code {
                KeyCode::Esc => app.enter_normal_mode(),
                KeyCode::Enter => {
                    // Process command - for now, just go back to normal mode
                    app.enter_normal_mode();
                },
                KeyCode::Backspace => {
                    if !app.search_query.is_empty() {
                        app.search_query.pop();
                    }
                },
                KeyCode::Char(c) => {
                    // Add character to search query
                    app.search_query.push(c);
                    
                    // Live search as you type
                    if let Some(SearchMode::Forward) = app.search_mode {
                        app.search_in_chat();
                    } else if let Some(SearchMode::Backward) = app.search_mode {
                        app.search_in_chat();
                    }
                },
                _ => {}
            }
        }
    }
    
    Ok(false)
}

/// Run the main application loop
pub async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut SeaOrmAppState,
    config: &mut AppConfig,
    store: &SeaOrmStore,
    oaiclient: &AzureOpenAIClient,
    mcp_manager: &mut Option<MCPServerManager>,
) -> Result<(), Box<dyn Error>> {
    let mut should_quit = false;
    
    while !should_quit {
        // Draw the UI
        terminal.draw(|f| ui::render(f, app))?;
        
        // Handle key events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                should_quit = handle_key_event(
                    key.code, app, config, store, oaiclient, mcp_manager
                ).await?;
            }
        }
    }
    
    Ok(())
}