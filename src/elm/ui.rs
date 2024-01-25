use super::ApplicationStateModel;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::error::Error;

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

fn create_menu() {}

/// Renders the UI based on the current state.
pub fn render<B>(
    terminal: &mut Terminal<B>,
    _state: &ApplicationStateModel,
) -> Result<(), Box<dyn Error>>
where
    B: Backend,
{
    terminal.draw(|frame| {
        frame.render_widget(
            Block::default().borders(Borders::all()).title("Main"),
            centered_rect(frame.size(), 35, 35),
        );
    })?;

    Ok(())
}
