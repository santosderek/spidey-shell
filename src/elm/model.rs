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
        }
    }

    pub fn add_history(&mut self, history: ChatCompletionMessage) {
        self.history.push(history);
    }
}
