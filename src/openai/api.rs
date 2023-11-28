use lazy_static::lazy_static;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{
    ChatCompletionMessage, ChatCompletionRequest, ChatCompletionResponse, MessageRole,
};
use openai_api_rs::v1::common::GPT4_1106_PREVIEW;
use openai_api_rs::v1::error::APIError;
use std::env;
use std::error::Error;
use std::sync::{Arc, RwLock, RwLockWriteGuard};

lazy_static! {
    pub static ref HISTORY: Arc<RwLock<Vec<ChatCompletionMessage>>> = Arc::new(RwLock::new(vec![]));
}

/// Get an OpenAI environment variable from the environment and returns a client
fn get_openai_client() -> Result<Client, Box<dyn Error>> {
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => return Err("Could not find OPENAI_API_KEY in environment".into()),
    };
    Ok(Client::new(api_key))
}

/// Fetch a completion from the OpenAI API
pub fn fetch_completion(prompt: &str) -> Result<ChatCompletionResponse, APIError> {
    let mut _history: RwLockWriteGuard<Vec<ChatCompletionMessage>> = HISTORY.write().unwrap();
    _history.push(ChatCompletionMessage {
        role: MessageRole::user,
        content: String::from(prompt),
        name: None,
        function_call: None,
    });

    let req = ChatCompletionRequest::new(GPT4_1106_PREVIEW.to_string(), _history.clone());

    _history.clear();
    _history.extend(req.messages.clone());

    match get_openai_client() {
        Ok(client) => client.chat_completion(req),
        Err(error) => panic!("Error trying to get OpenAI client: {}", error),
    }
}

