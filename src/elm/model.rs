use openai_api_rs::v1::chat_completion::ChatCompletionMessage;

pub struct ApplicationStateModel {
    pub history: Vec<ChatCompletionMessage>,
    /// Name of the history file currently loaded
    pub history_name: String,
    /// Available history files in the cache directory
    pub history_file_list: Vec<String>,
}

impl ApplicationStateModel {
    pub fn new() -> ApplicationStateModel {
        ApplicationStateModel {
            history: Vec::new(),
            history_file_list: Vec::new(),
            history_name: String::new(),
        }
    }

    pub fn add_history(&mut self, history: ChatCompletionMessage) {
        self.history.push(history);
    }

    // pub fn remove_history(&mut self, history: ChatCompletionMessage) {
    //     self.history.retain(|x| x != &history);
    // }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    // pub fn load_history_file(&mut self, filename: String) {
    //     self.history_name = filename;
    //     self.history = load_history(filename);
    // }
}

#[derive(Clone)]
pub enum Message {
    /// Load a history file
    LoadHistoryFile(String),
    /// Save the current history to a file
    SaveHistoryFile(String),
    /// Add a message to the history
    AddHistory(ChatCompletionMessage),
    /// Remove a message from the history
    RemoveHistory(ChatCompletionMessage),
    /// Clear the history
    ClearHistory,
    /// Update the history file list
    UpdateHistoryFileList,
    // Do Nothing
    NoOp,
}

pub fn update<'a>(model: &mut ApplicationStateModel, msg: &Message) -> Option<Message> {
    match msg {
        // Message::LoadHistoryFile(filename) => {
        //     model.history_name = filename;
        //     model.history = model.load_history_file(filename);
        //     msg
        // }

        // Message::SaveHistoryFile(filename) => {
        //     save_history(filename, model.history);
        //     msg
        // }
        Message::AddHistory(chat_message) => {
            model.add_history(chat_message.clone());
            Some(msg.clone())
        }
        // Message::RemoveHistory(message) => {
        //     model.remove_history(message);
        //     msg
        // }
        Message::ClearHistory => {
            model.clear_history();
            Some(msg.clone())
        }
        // Message::UpdateHistoryFileList => {
        //     model.history_file_list = get_history_file_list();
        //     msg
        // }
        _ => Some(msg.clone()),
    }
}
