use ratatui::{layout::Rect, Frame};
use tui_textarea::TextArea;

use crate::elm::ApplicationStateModel;

pub fn render(frame: &mut Frame<'_>, chunk: Rect, _state: &ApplicationStateModel) {
    let textarea = TextArea::default();
    let widget = textarea.widget();
    frame.render_widget(widget, chunk);
}
