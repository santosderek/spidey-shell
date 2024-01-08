use lazy_static::lazy_static;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{
    ChatCompletionMessage, ChatCompletionRequest, ChatCompletionResponse, MessageRole,
};
use openai_api_rs::v1::common::GPT4_1106_PREVIEW;
use openai_api_rs::v1::error::APIError;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::openai::history::History;

lazy_static! {
    pub static ref CURRENT_HISTORY_NAME: Arc<RwLock<String>> =
        Arc::new(RwLock::new(String::from("Untitled")));
}

async fn get_cache_directory() -> Result<PathBuf, Box<dyn Error>> {
    let cache_dir = match dirs::cache_dir() {
        Some(dir) => dir,
        None => return Err("Could not find cache directory".into()),
    };

    let cache_dir = cache_dir.join("spidey-shell");

    if !cache_dir.exists() {
        match tokio::fs::create_dir(cache_dir.clone()).await {
            Ok(_) => {}
            Err(_) => return Err("Could not create cache directory".into()),
        }
    }

    Ok(cache_dir)
}

/// Get an OpenAI environment variable from the environment and returns a client
fn get_openai_client() -> Result<Client, Box<dyn Error>> {
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => return Err("Could not find OPENAI_API_KEY in environment".into()),
    };
    Ok(Client::new(api_key))
}

pub async fn load_history_list() -> Result<Vec<String>, Box<dyn Error>> {
    let mut history_list: Vec<String> = vec![];

    let cache_dir = match get_cache_directory().await {
        Ok(dir) => dir,
        Err(e) => return Err(e),
    };

    let mut dir_entries = match tokio::fs::read_dir(cache_dir).await {
        Ok(entries) => entries,
        Err(_) => return Err("Could not read cache directory".into()),
    };

    while let Some(entry) = dir_entries.next_entry().await? {
        let file_name = entry.file_name();
        let file_name = file_name.to_str().unwrap();
        history_list.push(file_name.to_string());
    }

    Ok(history_list)
}

pub async fn load_history(name: String) -> Result<(), Box<dyn Error>> {
    let cache_dir = match get_cache_directory().await {
        Ok(dir) => dir,
        Err(e) => {
            println!("Could not get cache directory: {}", e);
            return Err(e);
        }
    };

    let history_file_path = cache_dir.join(name.clone());

    let history_file_content = match tokio::fs::read_to_string(history_file_path).await {
        Ok(content) => content,
        Err(_) => {
            println!("Could not open history file {}", name);
            return Err("Could not open history file".into());
        }
    };

    let history: History = match serde_json::from_str(&history_file_content) {
        Ok(history) => history,
        Err(_) => {
            println!("Could not parse history file content as JSON {}", name);
            return Err("Could not parse history file content as JSON".into());
        }
    };

    let mut history_buffer: Vec<ChatCompletionMessage> = vec![];
    history.messages.iter().for_each(|message| {
        history_buffer.push(message.to_chat_completion_message());
    });
    return Ok(());
}

/// Fetch a completion from the OpenAI API
pub fn fetch_completion(
    prompt: &str,
    mut history: History,
) -> Result<ChatCompletionResponse, APIError> {
    history.push_completion_message(ChatCompletionMessage {
        role: MessageRole::user,
        content: String::from(prompt),
        name: None,
        function_call: None,
    });

    let completion_message: Vec<ChatCompletionMessage> = history
        .messages
        .iter()
        .map(|message| message.to_chat_completion_message())
        .collect();

    let req = ChatCompletionRequest::new(GPT4_1106_PREVIEW.to_string(), completion_message);

    history.clear();
    history.extend(req.messages.clone());

    match get_openai_client() {
        Ok(client) => client.chat_completion(req),
        Err(error) => panic!("Error trying to get OpenAI client: {}", error),
    }
}
