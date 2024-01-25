use super::ApplicationStateModel;
use ratatui::{backend::Backend, widgets::Paragraph, Terminal};
use std::error::Error;

/// Renders the UI based on the current state.
pub fn render<B>(
    terminal: &mut Terminal<B>,
    _state: &ApplicationStateModel,
) -> Result<(), Box<dyn Error>>
where
    B: Backend,
{
    terminal.draw(|frame| {
        let area = frame.size();
        frame.render_widget(Paragraph::new("Hello Ratatui! (press 'q' to quit)"), area);
    })?;

    Ok(())
}
