use openai_api_rs::v1::chat_completion::ChatCompletionMessage;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum CurrentScreen {
    Menu,
    Chat,
    History,
}

#[derive(Default, Eq, PartialEq)]
pub enum RunningState {
    #[default]
    Running,
    Done,
}

mod menu {
    use ratatui::widgets::ListState;
    use ratatui::{
        layout::Rect,
        style::{Color, Modifier, Style},
        text::Text,
        widgets::{Block, Borders, List, ListItem},
        Frame,
    };

    use crate::elm::ApplicationStateModel;
    pub struct MenuState {
        pub items: Vec<String>,
        pub state: ListState,
    }

    impl MenuState {
        pub fn new(items: Vec<&str>) -> MenuState {
            let mut state = ListState::default();
            state.select(Some(0));

            let items = items.iter().map(|i| i.to_string()).collect();

            MenuState { items, state }
        }

        pub fn select_next(&mut self) {
            let selected = self.state.selected();
            let next = match selected {
                Some(i) => {
                    if i >= self.items.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(next));
        }

        pub fn select_previous(&mut self) {
            let selected = self.state.selected();
            let previous = match selected {
                Some(i) => {
                    if i == 0 {
                        self.items.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(previous));
        }

        pub fn render(&self, frame: &mut Frame<'_>, chunk: Rect, state: &ApplicationStateModel) {
            let (current_menu, title) = match state.current_screen {
                super::CurrentScreen::Menu => (&state.root_menu_state, "Main Menu"),
                super::CurrentScreen::Chat => (&state.chat_menu_state, "Chat"),
                super::CurrentScreen::History => (&state.history_menu_state, "History"),
            };
            let list: List = List::new(
                current_menu
                    .items
                    .iter()
                    .map(|i| ListItem::new(Text::from(i.as_str())))
                    .collect::<Vec<ListItem>>(),
            );

            frame.render_stateful_widget(
                list.block(Block::default().borders(Borders::ALL).title(title))
                    .highlight_style(
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::REVERSED)
                            .fg(Color::LightBlue),
                    ),
                chunk,
                &mut current_menu.state.clone(),
            );
        }
    }
}

pub struct ApplicationStateModel<'a> {
    /* The Global State of the Application */
    pub current_screen: CurrentScreen,
    pub running_state: RunningState,

    /* Menu Specific State */
    pub root_menu_state: menu::MenuState,
    pub chat_menu_state: menu::MenuState,
    pub history_menu_state: menu::MenuState,

    /* The OpenAI Chat API Specific State */
    pub history: Vec<ChatCompletionMessage>,
    /// Name of the history file currently loaded
    pub history_name: String,
    /// Available history files in the cache directory
    pub history_file_list: Vec<String>,
    // The text area for the chat
    pub chat_text_area: tui_textarea::TextArea<'a>,
}

impl ApplicationStateModel<'_> {
    pub fn new<'a>() -> ApplicationStateModel<'a> {
        ApplicationStateModel {
            current_screen: CurrentScreen::Menu,
            history: Vec::new(),
            history_file_list: Vec::new(),
            history_name: String::new(),
            running_state: RunningState::Running,
            root_menu_state: menu::MenuState::new(vec!["Chat", "History", "Quit"]),
            chat_menu_state: menu::MenuState::new(vec!["Send Message", "Back"]),
            history_menu_state: menu::MenuState::new(vec!["See History", "Back"]),
            chat_text_area: tui_textarea::TextArea::default(),
        }
    }

    pub fn add_history(&mut self, history: ChatCompletionMessage) {
        self.history.push(history);
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

#[derive(Clone)]
pub enum MenuMessage {
    SelectItem,
    SelectNext,
    SelectPrevious,
    NoOp,
}

#[derive(Clone)]
pub enum ChatMessage {
    AddMessage(ChatCompletionMessage),
    RemoveMessage(ChatCompletionMessage),
    ClearMessages,
    NoOp,
}

#[derive(Clone)]
pub enum EventMessage {
    MenuAction(MenuMessage),
    NoOp,
}

pub fn update<'a>(
    state: &'a mut ApplicationStateModel,
    msg: &EventMessage,
) -> Option<EventMessage> {
    let get_state_for_current_screen =
        |state: &'a mut ApplicationStateModel| -> &'a mut menu::MenuState {
            match state.current_screen {
                CurrentScreen::Menu => &mut state.root_menu_state,
                CurrentScreen::Chat => &mut state.chat_menu_state,
                CurrentScreen::History => &mut state.history_menu_state,
            }
        };

    match msg {
        EventMessage::MenuAction(action) => {
            match action {
                MenuMessage::SelectNext => {
                    get_state_for_current_screen(state).select_next();
                }

                MenuMessage::SelectPrevious => {
                    get_state_for_current_screen(state).select_previous();
                }

                MenuMessage::SelectItem if state.current_screen == CurrentScreen::Menu => {
                    match state.root_menu_state.state.selected() {
                        Some(0) => {
                            state.current_screen = CurrentScreen::Chat;
                        }
                        Some(1) => {
                            state.current_screen = CurrentScreen::History;
                        }
                        Some(2) => {
                            state.running_state = RunningState::Done;
                        }
                        _ => {}
                    }
                }
                MenuMessage::SelectItem if state.current_screen == CurrentScreen::Chat => {
                    match state.chat_menu_state.state.selected() {
                        Some(0) => {
                            // Send Message
                        }
                        Some(1) => {
                            state.chat_menu_state.state.select(Some(0));
                            state.current_screen = CurrentScreen::Menu;
                        }
                        _ => {}
                    }
                }
                MenuMessage::SelectItem if state.current_screen == CurrentScreen::History => {
                    match state.history_menu_state.state.selected() {
                        Some(0) => {
                            // See History
                        }
                        Some(1) => {
                            state.history_menu_state.state.select(Some(0));
                            state.current_screen = CurrentScreen::Menu;
                        }
                        _ => {}
                    }
                }
                MenuMessage::NoOp => {}
                _ => {}
            }

            Some(msg.clone())
        }

        _ => Some(msg.clone()),
    }
}
