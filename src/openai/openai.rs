use lazy_static::lazy_static;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{
    ChatCompletionMessage, ChatCompletionRequest, ChatCompletionResponse, MessageRole,
};
use openai_api_rs::v1::common::GPT4_1106_PREVIEW;
use openai_api_rs::v1::error::APIError;
use std::env;
use std::sync::{Mutex, MutexGuard, RwLock};

lazy_static! {
    pub static ref HISTORY: Mutex<Vec<ChatCompletionMessage>> = Mutex::new(vec![]);
}
// TODO: Need to make this return a Result
fn get_openai_client() -> Client {
    let api_key = env::var("OPENAI_API_KEY").unwrap();
    Client::new(api_key)
}

/// Fetch a completion from the OpenAI API
pub fn fetch_completion(prompt: &str) -> Result<ChatCompletionResponse, APIError> {
    let mut _history: MutexGuard<Vec<ChatCompletionMessage>> = HISTORY.lock().unwrap();
    _history.push(ChatCompletionMessage {
        role: MessageRole::user,
        content: String::from(prompt),
        name: None,
        function_call: None,
    });

    let req = ChatCompletionRequest::new(GPT4_1106_PREVIEW.to_string(), _history.clone());

    _history.clear();
    _history.extend(req.messages.clone());

    get_openai_client().chat_completion(req)
}
