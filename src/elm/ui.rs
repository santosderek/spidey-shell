use super::{model::CurrentScreen, ApplicationStateModel};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Renders the UI based on the current state.
pub fn render<'a, B>(
    terminal: &mut Terminal<B>,
    state: &'a mut ApplicationStateModel,
) -> &'a mut ApplicationStateModel
where
    B: Backend,
{
    let result = terminal.draw(|frame| {
        let size = frame.size();
        let chunks = centered_rect(size, 50, 50);

        match state.current_screen {
            CurrentScreen::Menu => {
                state.root_menu_state.render(frame, chunks, state);
            }
            CurrentScreen::Chat => {
                state.chat_menu_state.render(frame, chunks, state);
            }
            CurrentScreen::History => {
                state.history_menu_state.render(frame, chunks, state);
            }
        }
    });

    match result {
        Ok(_) => state,
        Err(e) => panic!("Error rendering the UI: {}", e),
    }
}
