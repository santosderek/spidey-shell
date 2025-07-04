use async_openai::{
    config::AzureConfig,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct AzureOpenAIConfig {
    pub api_key: String,
    pub api_base: String,
    pub deployment_id: String,
    pub api_version: String,
    pub model: String,
}

impl AzureOpenAIConfig {
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let api_key = env::var("AZURE_OPENAI_API_KEY")
            .or_else(|_| env::var("AZURE_OPENAI_KEY"))
            .map_err(|_| "Missing AZURE_OPENAI_API_KEY or AZURE_OPENAI_KEY environment variable")?;
        let api_base = env::var("AZURE_OPENAI_ENDPOINT")
            .map_err(|_| "Missing AZURE_OPENAI_ENDPOINT environment variable")?;
        let deployment_id = env::var("AZURE_OPENAI_DEPLOYMENT_ID")
            .map_err(|_| "Missing AZURE_OPENAI_DEPLOYMENT_ID environment variable")?;
        // Use a fixed known working API version to avoid 404 errors
        // Override whatever is set in the environment since the tests confirmed
        // that 2024-02-15-preview works with the current endpoint
        let api_version = "2024-02-15-preview".to_string();
        let model = env::var("AZURE_OPENAI_MODEL").unwrap_or_else(|_| "gpt-4.1".to_string());
        Ok(Self {
            api_key,
            api_base,
            deployment_id,
            api_version,
            model,
        })
    }
}

pub struct AzureOpenAIClient {
    client: Client<AzureConfig>,
    config: AzureOpenAIConfig,
}

impl AzureOpenAIClient {
    pub fn new(config: AzureOpenAIConfig) -> Self {
        let azure_config = AzureConfig::new()
            .with_api_base(&config.api_base)
            .with_api_key(&config.api_key)
            .with_deployment_id(&config.deployment_id)
            .with_api_version(&config.api_version);
        Self {
            client: Client::with_config(azure_config),
            config,
        }
    }

    pub async fn send_chat(&self, history: &[Message]) -> Result<Message, Box<dyn Error>> {
        let mut messages = Vec::new();
        for m in history {
            match m.role.as_str() {
                "system" => {
                    messages.push(
                        ChatCompletionRequestSystemMessageArgs::default()
                            .content(m.content.clone())
                            .build()?
                            .into(),
                    );
                }
                "user" => {
                    messages.push(
                        ChatCompletionRequestUserMessageArgs::default()
                            .content(m.content.clone())
                            .build()?
                            .into(),
                    );
                }
                _ => {}
            }
        }
        let req = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u32)
            .model(&self.config.model)
            .messages(messages)
            .build()?;
        let resp = self.client.chat().create(req).await?;
        let content = resp
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();
        Ok(Message {
            role: "assistant".into(),
            content,
        })
    }
}
