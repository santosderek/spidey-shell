use openai_api_rs::v1::chat_completion::ChatCompletionMessage;
use ratatui::widgets::ListState;

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

pub struct MenuState {
    pub items: Vec<String>,
    pub state: ListState,
}

impl MenuState {
    pub fn new() -> MenuState {
        let mut state = ListState::default();
        state.select(Some(0));

        MenuState {
            items: vec![
                "Chat".to_string(),
                "History".to_string(),
                "Quit".to_string(),
            ],
            state,
        }
    }
}
impl MenuState {
    fn select_next(&mut self) {
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

    fn select_previous(&mut self) {
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
}

pub struct ApplicationStateModel {
    /* The Global State of the Application */
    pub current_screen: CurrentScreen,
    pub running_state: RunningState,

    /* Menu Specific State */
    pub menu_state: MenuState,

    /* The OpenAI Chat API Specific State */
    pub history: Vec<ChatCompletionMessage>,
    /// Name of the history file currently loaded
    pub history_name: String,
    /// Available history files in the cache directory
    pub history_file_list: Vec<String>,
}

impl ApplicationStateModel {
    pub fn new() -> ApplicationStateModel {
        ApplicationStateModel {
            current_screen: CurrentScreen::Menu,
            history: Vec::new(),
            history_file_list: Vec::new(),
            history_name: String::new(),
            running_state: RunningState::Running,
            menu_state: MenuState::new(),
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
    ChatAction(ChatMessage),
    // Do Nothing
    NoOp,
}

pub fn update<'a>(state: &mut ApplicationStateModel, msg: &EventMessage) -> Option<EventMessage> {
    match msg {
        EventMessage::MenuAction(action) => {
            match action {
                MenuMessage::SelectNext => {
                    state.menu_state.select_next();
                }
                MenuMessage::SelectPrevious => {
                    state.menu_state.select_previous();
                }

                MenuMessage::SelectItem => match state.menu_state.state.selected() {
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
                },
                MenuMessage::NoOp => {}
            }

            Some(msg.clone())
        }
        _ => Some(msg.clone()),
    }
}
