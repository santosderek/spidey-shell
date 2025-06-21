use spidey_shell::persistence::ConversationStore;
use spidey_shell::openai::Message;
use tempfile::TempDir;

#[test]
fn test_conversation_store_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    
    let store = ConversationStore::new(db_path);
    assert!(store.is_ok(), "Should be able to create conversation store");
}

#[test]
fn test_create_conversation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let store = ConversationStore::new(db_path).expect("Failed to create store");
    
    let conversation_id = store.create_conversation("Test Conversation");
    assert!(conversation_id.is_ok(), "Should be able to create conversation");
    
    let conversation_id = conversation_id.unwrap();
    assert!(conversation_id > 0, "Conversation ID should be positive");
}

#[test]
fn test_add_message() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let store = ConversationStore::new(db_path).expect("Failed to create store");
    
    let conversation_id = store.create_conversation("Test Conversation").expect("Failed to create conversation");
    
    let message = Message {
        role: "user".to_string(),
        content: "Hello, world!".to_string(),
    };
    
    let result = store.add_message(conversation_id, &message);
    assert!(result.is_ok(), "Should be able to add message to conversation");
}

