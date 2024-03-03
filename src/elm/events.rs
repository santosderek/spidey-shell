use super::model::{CurrentScreen, MenuMessage, RunningState};
use super::{render, update};
use super::{ApplicationStateModel, EventMessage};
use crossterm::event::{self, KeyCode};
use ratatui::{backend::Backend, Terminal};
use std::error::Error;
use std::time::Duration;

/// The main event loop which handles events and updates the state based on the current message.
pub async fn run_event_loop<B>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>>
where
    B: Backend,
{
    /* Basically the global state: */
    let mut state = ApplicationStateModel::new();

    while state.running_state != RunningState::Done {
        let mut state = render(terminal, &mut state);
        let mut message = EventMessage::NoOp;

        if event::poll(Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                match state.current_screen {
                    CurrentScreen::Menu | CurrentScreen::Chat | CurrentScreen::History => {
                        message = match key.code {
                            KeyCode::Char('h') | KeyCode::Left => {
                                EventMessage::MenuAction(MenuMessage::SelectPrevious)
                            }
                            KeyCode::Char('j') | KeyCode::Down => {
                                EventMessage::MenuAction(MenuMessage::SelectNext)
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                EventMessage::MenuAction(MenuMessage::SelectPrevious)
                            }
                            KeyCode::Char('l') | KeyCode::Right => {
                                EventMessage::MenuAction(MenuMessage::SelectNext)
                            }
                            KeyCode::Enter => EventMessage::MenuAction(MenuMessage::SelectItem),
                            _ => EventMessage::NoOp,
                        };
                    }
                }

                // global actions
                if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                    state.running_state = RunningState::Done;
                }
            }
        }

        update(&mut state, &message).unwrap();
    }
    return Ok(());
}
