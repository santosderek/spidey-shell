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
                let list: List = List::new(
                    state
                        .menu_state
                        .items
                        .iter()
                        .map(|i| ListItem::new(Text::from(i.as_str())))
                        .collect::<Vec<ListItem>>(),
                );

                frame.render_stateful_widget(
                    list.block(Block::default().borders(Borders::ALL).title("Menu"))
                        .highlight_style(
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .add_modifier(Modifier::REVERSED)
                                .fg(Color::LightBlue),
                        ),
                    chunks,
                    &mut state.menu_state.state,
                );
            }
            CurrentScreen::Chat => {
                let chat = Paragraph::new(Text::from("Chat"))
                    .block(Block::default().borders(Borders::ALL).title("Chat"))
                    .alignment(Alignment::Center);
                frame.render_widget(chat, chunks);
            }
            CurrentScreen::History => {
                let history = Paragraph::new(Text::from("History"))
                    .block(Block::default().borders(Borders::ALL).title("History"))
                    .alignment(Alignment::Center);

                frame.render_widget(history, chunks);
            }
        }
    });

    match result {
        Ok(_) => state,
        Err(e) => panic!("Error rendering the UI: {}", e),
    }
}
