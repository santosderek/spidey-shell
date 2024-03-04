use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
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
    let textarea = &state.chat_text_area;

    frame.render_widget(textarea.widget(), layout[1]);

    state.chat_menu_state.render(frame, layout[2], state);
}
