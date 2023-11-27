use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest, ChatCompletionResponse};
use openai_api_rs::v1::common::GPT4;
use openai_api_rs::v1::error::APIError;
use std::env;

// TODO: Need to make this return a Result
fn get_openai_client() -> Client {
    let api_key = env::var("OPENAI_API_KEY").unwrap();
    Client::new(api_key)
}

pub fn fetch_completion(prompt: &str) -> Result<ChatCompletionResponse, APIError> {
    let req = ChatCompletionRequest::new(
        GPT4.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: String::from(prompt),
            name: None,
            function_call: None,
        }],
    );

    get_openai_client().chat_completion(req)
}
