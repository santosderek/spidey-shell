use super::{render, update};
use super::{ApplicationStateModel, Message};
use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{backend::Backend, Terminal};
use std::error::Error;
use std::time::Duration;

/// The main event loop which handles events and updates the state based on the current message.
pub async fn run_event_loop<B>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>>
where
    B: Backend,
{
    let mut state = ApplicationStateModel::new();
    let mut message = Message::NoOp;

    loop {
        let _ = render(terminal, &state);

        if event::poll(Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break Ok(());
                }
            }
        }

        message = update(&mut state, &message).unwrap();
    }
}
