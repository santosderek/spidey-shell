use async_openai::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

pub async fn send_chat(history: &[Message]) -> Result<Message, Box<dyn Error>> {
    let client = Client::new();
    let msgs: Vec<_> = history
        .iter()
        .map(|m| json!({"role": m.role, "content": m.content}))
        .collect();
    let req = json!({
        "model": "gpt-4o",
        "messages": msgs
    });
    let resp: serde_json::Value = client.chat().create_byot(req).await?;
    let content = resp["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();
    Ok(Message {
        role: "assistant".into(),
        content,
    })
}
