use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use super::models::{InputMode, SearchMode};
use super::state::AppState;

/// Main UI rendering function
pub fn render(f: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(f.area());

    // Defensive check: only render if there are at least 2 chunks
    if chunks.len() >= 2 {
        render_sidebar(f, app, chunks[0]);
        render_main_area(f, app, chunks[1]);
    }
}

/// Render the sidebar with conversations list and MCP servers
fn render_sidebar(f: &mut Frame, app: &AppState, area: Rect) {
    // Split sidebar into conversations list and MCP server area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(8)])
        .split(area);

    render_conversations_list(f, app, chunks[0]);

    if !app.mcp_servers.is_empty() {
        render_mcp_server_list(f, app, chunks[1]);
    }
}

/// Render the conversations list
fn render_conversations_list(f: &mut Frame, app: &AppState, area: Rect) {
    let items: Vec<ListItem> = app
        .conversations_list
        .iter()
        .enumerate()
        .map(|(index, s)| {
            let style = if app.current_conversation() == *s {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            // Style special items differently
            let (display_text, item_style) = if index < 2 {
                // Special items
                match index {
                    0 => (
                        s.to_string(),
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    ),
                    1 => (s.to_string(), Style::default().fg(Color::Cyan)),
                    _ => unreachable!(),
                }
            } else if let Some(conv_id) = app.get_conversation_id_by_index(index) {
                // Regular conversation with timestamp
                if let Some(conv) = app.conversations.iter().find(|c| c.id == conv_id) {
                    let datetime = chrono::DateTime::from_timestamp(conv.created_at, 0)
                        .map(|dt| format!("{}", dt.format("%Y-%m-%d %H:%M")))
                        .unwrap_or_else(|| "Unknown date".to_string());

                    (format!("{} ({})", s, datetime), style)
                } else {
                    (s.to_string(), style)
                }
            } else {
                (s.to_string(), style)
            };

            ListItem::new(display_text).style(item_style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Conversations"),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.state.clone());
}

/// Render the MCP server list
fn render_mcp_server_list(f: &mut Frame, app: &AppState, area: Rect) {
    // Sort servers by active status first, then by name
    let mut sorted_servers: Vec<(&String, &bool)> = app.mcp_servers.iter().collect();
    sorted_servers.sort_by(|a, b| {
        let a_active = app.active_mcp_servers.contains(a.0);
        let b_active = app.active_mcp_servers.contains(b.0);

        // First sort by active status (active first)
        if a_active && !b_active {
            return std::cmp::Ordering::Less;
        } else if !a_active && b_active {
            return std::cmp::Ordering::Greater;
        }

        // Then sort by name
        a.0.cmp(b.0)
    });

    let mcp_items: Vec<ListItem> = sorted_servers
        .iter()
        .map(|(name, &enabled)| {
            let is_active = app.active_mcp_servers.contains(*name);

            // Different prefixes for active/inactive servers
            let prefix = if is_active { "[✓] " } else { "[ ] " };

            // Add a symbol to show server status
            let status_symbol = if enabled {
                "● " // Connected
            } else {
                "○ " // Disconnected
            };

            // Different styles for active/inactive servers
            let style = if enabled && is_active {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else if enabled {
                Style::default().fg(Color::White)
            } else if is_active {
                Style::default().fg(Color::Yellow) // Warning - active but disabled
            } else {
                Style::default().fg(Color::DarkGray)
            };

            ListItem::new(format!("{}{}{}", prefix, status_symbol, name)).style(style)
        })
        .collect();

    // Create a list title with counts
    let active_count = app.active_mcp_servers.len();
    let total_count = app.mcp_servers.len();
    let title = format!("MCP Servers [{}/{}]", active_count, total_count);

    let mcp_list = List::new(mcp_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(if active_count > 0 {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            }),
    );

    f.render_widget(mcp_list, area);

    // Add instructions below MCP servers if there's space
    if area.height > app.mcp_servers.len() as u16 + 3 {
        let help_text =
            "Press 'M' to toggle server\nUse '@server' in messages\nEsc for normal mode";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Left);

        // Create a small area below the MCP servers list
        let help_area = Rect {
            x: area.x + 1,
            y: area.y + app.mcp_servers.len() as u16 + 2,
            width: area.width - 2,
            height: area.height - app.mcp_servers.len() as u16 - 3,
        };

        if help_area.height >= 3 {
            f.render_widget(help, help_area);
        }
    }
}

/// Render the main chat area and input box
fn render_main_area(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    render_chat_history(f, app, chunks[0]);
    render_input_area(f, app, chunks[1]);
}

/// Render the chat message history
fn render_chat_history(f: &mut Frame, app: &AppState, area: Rect) {
    // Get current messages
    let current_messages = app.current_messages();
    let messages: Vec<ListItem> = current_messages
        .iter()
        .enumerate()
        .map(|(idx, titled_msg)| {
            let srv = &titled_msg.title;
            let m = &titled_msg.message;

            let line = if app.current_conversation() == "All Conversations" {
                format!("[{}] {}: {}", srv, m.role, m.content)
            } else {
                format!("{}: {}", m.role, m.content)
            };

            // Highlight the current selected message in normal mode
            let style = if app.input_mode == InputMode::Normal && idx == app.chat_scroll_position {
                Style::default().fg(Color::Yellow).bg(Color::DarkGray)
            } else {
                match m.role.as_str() {
                    "assistant" => Style::default().fg(Color::Green),
                    "user" => Style::default().fg(Color::White),
                    "system" => Style::default().fg(Color::Cyan),
                    "error" => Style::default().fg(Color::Red),
                    _ => Style::default(),
                }
            };

            ListItem::new(line).style(style)
        })
        .collect();

    // Build the history title showing active MCP servers and navigation info
    let mut history_title = "History".to_string();
    if !app.active_mcp_servers.is_empty() {
        history_title = format!(
            "{} (MCP: {})",
            history_title,
            app.active_mcp_servers.join(", ")
        );
    }

    // Add scrolling position information
    if !current_messages.is_empty() {
        history_title = format!(
            "{} [{}/{}]",
            history_title,
            app.chat_scroll_position + 1,
            current_messages.len()
        );
    }

    // Create list state with correct selection if in normal mode
    let mut list_state = ListState::default();
    if app.input_mode == InputMode::Normal && !current_messages.is_empty() {
        list_state.select(Some(app.chat_scroll_position));
    }

    let history = List::new(messages)
        .block(Block::default().borders(Borders::ALL).title(history_title))
        .highlight_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));

    f.render_stateful_widget(history, area, &mut list_state);
}

/// Render the input area with mode indicator
fn render_input_area(f: &mut Frame, app: &AppState, area: Rect) {
    // Create mode string
    let mode_str = match app.input_mode {
        InputMode::Normal => "NORMAL",
        InputMode::Insert => "INSERT",
        InputMode::Visual => "VISUAL",
        InputMode::Command => "COMMAND",
    };

    // Create search string if in search mode
    let search_str = if let Some(mode) = app.search_mode {
        match mode {
            SearchMode::Forward => format!("/ {}", app.search_query),
            SearchMode::Backward => format!("? {}", app.search_query),
        }
    } else {
        String::new()
    };

    // Create input title with appropriate instructions
    let input_title = if !search_str.is_empty() {
        format!(
            "{} - {} (Esc to cancel, Enter to search)",
            mode_str, search_str
        )
    } else if app.input_mode == InputMode::Normal {
        format!("{} - (j/k:scroll, /,?:search, i:insert, q:quit)", mode_str)
    } else {
        format!(
            "{} - (Enter to send, Esc for normal mode, q to quit, @server to activate)",
            mode_str
        )
    };

    // Set input style based on mode
    let input_style = Style::default().fg(match app.input_mode {
        InputMode::Normal => Color::Yellow,
        InputMode::Insert => Color::Green,
        InputMode::Visual => Color::Magenta,
        InputMode::Command => Color::Cyan,
    });

    // Create the input paragraph widget
    let input = Paragraph::new(app.input.as_str())
        .style(input_style)
        .block(Block::default().borders(Borders::ALL).title(input_title));

    f.render_widget(input, area);
}
