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
    let mut state = &mut ApplicationStateModel::new();

    while state.running_state != RunningState::Done {
        state = render(terminal, state);
        let mut message = EventMessage::NoOp;

        if event::poll(Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                match state.current_screen {
                    CurrentScreen::Menu | CurrentScreen::History => {
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

                    CurrentScreen::Chat => match (state.in_chat_area, key.code, key.modifiers) {
                        (_, KeyCode::Char('j'), event::KeyModifiers::CONTROL)
                        | (_, KeyCode::Char('k'), event::KeyModifiers::CONTROL) => {
                            state.in_chat_area = !state.in_chat_area;
                        }
                        (false, KeyCode::Char('j'), _) => {
                            message = EventMessage::MenuAction(MenuMessage::SelectNext);
                        }
                        (false, KeyCode::Char('k'), _) => {
                            message = EventMessage::MenuAction(MenuMessage::SelectPrevious);
                        }
                        (false, KeyCode::Enter, _) => {
                            message = EventMessage::MenuAction(MenuMessage::SelectItem);
                        }
                        (true, KeyCode::Esc, _) => {
                            state.in_chat_area = false;
                        }
                        (true, _, _) => {
                            state.chat_text_area.input(key);
                        }
                        _ => {}
                    },
                }

                // exit if CTRL + q is pressed
                if key.modifiers == event::KeyModifiers::CONTROL && key.code == KeyCode::Char('q') {
                    state.running_state = RunningState::Done;
                }
            }
        }

        update(&mut state, &message).unwrap();
    }
    return Ok(());
}
