use openai_api_rs::v1::chat_completion::{ChatCompletionMessage, MessageRole};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct History {
    pub messages: Vec<HistoryMessage>,
    name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryMessage {
    role: MessageRole,
    content: String,
}

impl History {
    pub fn new() -> History {
        History {
            messages: vec![],
            name: String::from("history"),
        }
    }

    /// Add a message to the history
    pub fn push_message(&mut self, message: HistoryMessage) {
        self.messages.push(message);
    }

    pub fn push_completion_message(&mut self, message: ChatCompletionMessage) {
        let history_message = HistoryMessage::new(message.role, message.content);
        self.messages.push(history_message);
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn extend(&mut self, messages: Vec<ChatCompletionMessage>) {
        messages.iter().for_each(|message| {
            self.push_completion_message(message.clone());
        });
    }
}

impl HistoryMessage {
    pub fn new(role: MessageRole, content: String) -> HistoryMessage {
        HistoryMessage { role, content }
    }

    /// Convert a HistoryMessage to a ChatCompletionMessage
    pub fn to_chat_completion_message(&self) -> ChatCompletionMessage {
        ChatCompletionMessage {
            role: self.role.clone(),
            content: self.content.clone(),
            name: None,
            function_call: None,
        }
    }
}

impl From<ChatCompletionMessage> for HistoryMessage {
    fn from(message: ChatCompletionMessage) -> Self {
        HistoryMessage {
            role: message.role,
            content: message.content,
        }
    }
}

impl From<HistoryMessage> for ChatCompletionMessage {
    fn from(message: HistoryMessage) -> Self {
        ChatCompletionMessage {
            role: message.role,
            content: message.content,
            name: None,
            function_call: None,
        }
    }
}
