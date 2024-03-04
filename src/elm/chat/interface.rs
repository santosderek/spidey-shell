use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Widget},
    Frame,
};

use crate::elm::ApplicationStateModel;

pub fn render(frame: &mut Frame<'_>, chunk: Rect, state: &ApplicationStateModel) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(20),
            Constraint::Percentage(10),
        ])
        .split(chunk);

    /* For the sake of rendering, we will clone it, then we will mutate it. */
    let textarea = &mut state.chat_text_area.clone();

    let chat = Block::default().title("Message").borders(Borders::ALL);
    textarea.set_block(chat);

    /* If we are in the chat area, we will change the cursor style. */
    if state.in_chat_area {
        textarea.set_cursor_style(Style::default().bg(ratatui::style::Color::White));
    } else {
        textarea.set_cursor_style(Style::default().bg(ratatui::style::Color::DarkGray));
    }

    frame.render_widget(textarea.widget(), layout[1]);
    state.chat_menu_state.render(frame, layout[2], state);
}
