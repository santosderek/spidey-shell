use spidey_shell::openai::{AzureOpenAIClient, AzureOpenAIConfig, Message};
use tracing::{debug, error, info, warn};

#[test]
fn test_config_from_env() {
    // Test that we can load config from environment
    let config = AzureOpenAIConfig::from_env();
    assert!(config.is_ok(), "Should be able to load config from environment");
    
    let config = config.unwrap();
    assert!(!config.api_key.is_empty(), "API key should not be empty");
    assert!(!config.api_base.is_empty(), "API base should not be empty");
    assert!(!config.deployment_id.is_empty(), "Deployment ID should not be empty");
    assert!(!config.api_version.is_empty(), "API version should not be empty");
    assert!(!config.model.is_empty(), "Model should not be empty");
    
    info!("✅ Config loaded successfully:");
    info!("   API Base: {}", config.api_base);
    info!("   Deployment ID: {}", config.deployment_id);
    info!("   API Version: {}", config.api_version);
    info!("   Model: {}", config.model);
    info!("   API Key: {}...{}", &config.api_key[..4], &config.api_key[config.api_key.len()-4..]);
}

#[test]
fn test_client_creation() {
    let config = AzureOpenAIConfig::from_env().expect("Config should load from environment");
    let _client = AzureOpenAIClient::new(config.clone());
    
    // Just verify the client was created successfully without panicking
    info!("✅ Client created successfully");
}

#[test]
fn test_message_serialization() {
    let message = Message {
        role: "user".to_string(),
        content: "Hello, world!".to_string(),
    };
    
    let json = serde_json::to_string(&message).expect("Should serialize to JSON");
    let deserialized: Message = serde_json::from_str(&json).expect("Should deserialize from JSON");
    
    assert_eq!(message.role, deserialized.role);
    assert_eq!(message.content, deserialized.content);
    
    info!("✅ Message serialization works correctly");
}

#[tokio::test]
async fn test_api_endpoint_connectivity() {
    let config = match AzureOpenAIConfig::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("❌ Skipping connectivity test: missing Azure API config: {}", e);
            return;
        }
    };
    
    info!("🔍 Testing API endpoint connectivity...");
    info!("   Endpoint: {}", config.api_base);
    info!("   Deployment: {}", config.deployment_id);
    info!("   API Version: {}", config.api_version);
    
    // Create a simple HTTP client to test the endpoint
    let client = reqwest::Client::new();
    
    // Test different API versions to find a working one
    let api_versions = vec![
        "2023-12-01-preview",
        "2024-02-01", 
        "2024-02-15-preview",
        "2023-05-15",
        &config.api_version
    ];
    
    let mut found_working_version = false;
    
    for api_version in &api_versions {
        let url = format!("{}/openai/deployments/{}/chat/completions?api-version={}", 
            config.api_base.trim_end_matches('/'),
            config.deployment_id,
            api_version
        );
        
        debug!("   Trying API version: {} -> {}", api_version, url);
        
        // Test with a HEAD request first to check if the endpoint exists
        let response = client
            .head(&url)
            .header("api-key", &config.api_key)
            .send()
            .await;
            
        match response {
            Ok(resp) => {
                debug!("     Status: {}", resp.status());
                if resp.status().is_success() || resp.status() == 405 {
                    // 405 Method Not Allowed is OK - means endpoint exists but HEAD not supported
                    info!("     ✅ Endpoint exists with API version: {}", api_version);
                    found_working_version = true;
                    break;
                } else if resp.status() == 404 {
                    warn!("     ❌ 404 Not Found");
                } else if resp.status() == 401 {
                    error!("     ❌ 401 Unauthorized - Check API key");
                    break;
                } else {
                    warn!("     ❌ Status: {}", resp.status());
                }
            }
            Err(e) => {
                error!("     ❌ Connection failed: {}", e);
            }
        }
    }
    
    if !found_working_version {
        error!("❌ No working API version found. Possible issues:");
        error!("   - Deployment ID '{}' might not exist", config.deployment_id);
        error!("   - Endpoint URL might be incorrect");
        error!("   - API key might be invalid");
        error!("");
        info!("💡 Suggestions:");
        info!("   1. Check your Azure OpenAI resource for the correct deployment name");
        info!("   2. Common deployment names: gpt-35-turbo, gpt-4, gpt-4-turbo");
        info!("   3. Verify the endpoint URL in Azure portal");
    }
}

#[tokio::test]
async fn test_send_chat_inference() {
    let config = match AzureOpenAIConfig::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("❌ Skipping inference test: missing Azure API config: {}", e);
            return;
        }
    };
    
    info!("🧪 Testing chat inference...");
    let client = AzureOpenAIClient::new(config);
    let prompt = vec![Message {
        role: "user".to_string(),
        content: "Say hello in one word".to_string(),
    }];
    
    info!("   Sending prompt: {:?}", prompt);
    
    let resp = client.send_chat(&prompt).await;
    match resp {
        Ok(msg) => {
            info!("✅ API call successful!");
            info!("   Response role: {}", msg.role);
            info!("   Response content: {}", msg.content);
            assert_eq!(msg.role, "assistant");
            assert!(!msg.content.trim().is_empty(), "Response should not be empty");
        }
        Err(e) => {
            error!("❌ API call failed: {}", e);
            error!("   This indicates a configuration or API issue");
            panic!("Chat inference test failed: {}", e);
        }
    }
}

#[tokio::test] 
async fn test_send_chat_with_system_message() {
    let config = match AzureOpenAIConfig::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("❌ Skipping system message test: missing Azure API config: {}", e);
            return;
        }
    };
    
    info!("🧪 Testing chat with system message...");
    let client = AzureOpenAIClient::new(config);
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: "You are a helpful assistant that responds in exactly 3 words.".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: "What is AI?".to_string(),
        }
    ];
    
    info!("   Sending messages: {:?}", messages);
    
    let resp = client.send_chat(&messages).await;
    match resp {
        Ok(msg) => {
            info!("✅ System message test successful!");
            info!("   Response: {}", msg.content);
            assert_eq!(msg.role, "assistant");
            assert!(!msg.content.trim().is_empty(), "Response should not be empty");
        }
        Err(e) => {
            error!("❌ System message test failed: {}", e);
            panic!("System message test failed: {}", e);
        }
    }
}

