use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};


mod openai;
mod persistence;
use openai::{AzureOpenAIClient, AzureOpenAIConfig, Message};
use persistence::config::AppConfig;
use persistence::ConversationStore;

struct App {
    servers: Vec<String>,
    state: ListState,
    input: String,
    messages: HashMap<String, Vec<Message>>, // server -> messages
    current_conversation_id: Option<i64>,
}

impl App {
    fn new() -> Self {
        let servers = load_servers();
        let mut messages = HashMap::new();
        for s in &servers {
            if s == "All" { continue; }
            messages.insert(s.clone(), load_history(s).unwrap_or_default());
        }
        let mut state = ListState::default();
        if !servers.is_empty() { state.select(Some(0)); }
        Self { servers, state, input: String::new(), messages, current_conversation_id: None }
    }

    fn current_server(&self) -> String {
        self.servers
            .get(self.state.selected().unwrap_or(0))
            .cloned()
            .unwrap_or_else(|| "All".to_string())
    }

    fn current_messages(&self) -> Vec<(String, Message)> {
        let server = self.current_server();
        if server == "All" {
            let mut res = Vec::new();
            for (srv, msgs) in &self.messages {
                for m in msgs {
                    res.push((srv.clone(), m.clone()));
                }
            }
            res
        } else {
            self
                .messages
                .get(&server)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|m| (server.clone(), m))
                .collect()
        }
    }
}

fn load_servers() -> Vec<String> {
    let mut servers = Vec::new();
    if let Ok(read) = fs::read_dir("mcp_servers") {
        for entry in read.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let name = entry.file_name().to_string_lossy().into_owned();
                servers.push(name);
            }
        }
    }
    servers.sort();
    // Place 'None' first, then 'All', then the rest
    let mut list = vec!["None".to_string(), "All".to_string()];
    list.extend(servers);
    list
}

fn load_history(server: &str) -> io::Result<Vec<Message>> {
    let path = format!("mcp_servers/{}/history.json", server);
    if Path::new(&path).exists() {
        let data = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data).unwrap_or_default())
    } else {
        Ok(Vec::new())
    }
}

fn save_history(server: &str, msgs: &[Message]) -> io::Result<()> {
    let dir = format!("mcp_servers/{}", server);
    fs::create_dir_all(&dir)?;
    let path = format!("{}/history.json", dir);
    fs::write(path, serde_json::to_string_pretty(msgs).unwrap())?;
    Ok(())
}

// Navigation Domain - handles server/conversation selection
fn handle_navigation_up(app: &mut App) {
    if let Some(selected) = app.state.selected() {
        let new = selected.saturating_sub(1);
        app.state.select(Some(new));
    }
}

fn handle_navigation_down(app: &mut App) {
    if let Some(selected) = app.state.selected() {
        let new = (selected + 1).min(app.servers.len().saturating_sub(1));
        app.state.select(Some(new));
    }
}

// Conversation Management Domain - handles conversation creation and management
fn handle_conversation_creation(
    app: &mut App,
    config: &mut AppConfig,
    store: &ConversationStore,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut server = app.current_server();
    
    if server == "None" {
        // DB-backed: create new conversation
        let conv_title = format!("conv_{}", chrono::Utc::now().timestamp());
        let conversation_id = store.create_conversation(&conv_title)?;
        config.conversation_id = Some(conversation_id);
        config.save()?;
        app.current_conversation_id = Some(conversation_id);
        
        // Add pseudo-server
        app.servers.push(conv_title.clone());
        app.state.select(Some(app.servers.len() - 1));
        app.messages.insert(conv_title.clone(), Vec::new());
        server = conv_title;
    }
    
    if server == "All" {
        server = app.servers
            .iter()
            .find(|s| *s != "All" && *s != "None")
            .cloned()
            .unwrap_or_else(|| "".to_string());
    }
    
    Ok(server)
}

// Message Processing Domain - handles chat message processing
async fn handle_message_processing(
    app: &mut App,
    server: &str,
    oaiclient: &AzureOpenAIClient,
) -> Result<(), Box<dyn std::error::Error>> {
    if !server.is_empty() && !app.input.is_empty() {
        let msgs = app.messages.entry(server.to_string()).or_default();
        msgs.push(Message {
            role: "user".into(),
            content: app.input.clone(),
        });
        
        let reply = match oaiclient.send_chat(msgs).await {
            Ok(r) => r,
            Err(e) => Message {
                role: "error".into(),
                content: format!("{}", e),
            },
        };
        
        msgs.push(reply);
        app.input.clear();
    }
    
    Ok(())
}

// Persistence Domain - handles saving conversation data
fn handle_persistence(
    app: &App,
    server: &str,
    store: &ConversationStore,
) -> Result<(), Box<dyn std::error::Error>> {
    let empty_vec = Vec::new();
    let msgs = app.messages.get(server).unwrap_or(&empty_vec);
    
    if let Some(cid) = app.current_conversation_id {
        for m in msgs {
            let _ = store.add_message(cid, m);
        }
    } else {
        let _ = save_history(server, msgs);
    }
    
    Ok(())
}

// Input Processing Domain - handles text input
fn handle_text_input(app: &mut App, c: char) {
    app.input.push(c);
}

fn handle_backspace(app: &mut App) {
    app.input.pop();
}

// Event Handling Domain - processes keyboard events
async fn handle_key_event(
    key_code: KeyCode,
    app: &mut App,
    config: &mut AppConfig,
    store: &ConversationStore,
    oaiclient: &AzureOpenAIClient,
) -> Result<bool, Box<dyn std::error::Error>> {
    match key_code {
        KeyCode::Char('q') => return Ok(true), // Signal to quit
        KeyCode::Up => handle_navigation_up(app),
        KeyCode::Down => handle_navigation_down(app),
        KeyCode::Enter => {
            let server = handle_conversation_creation(app, config, store)?;
            handle_message_processing(app, &server, oaiclient).await?;
            handle_persistence(app, &server, store)?;
        }
        KeyCode::Backspace => handle_backspace(app),
        KeyCode::Char(c) => handle_text_input(app, c),
        _ => {}
    }
    Ok(false) // Continue running
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    
    // Check if we're running in a proper terminal
    let has_tty = crossterm::tty::IsTty::is_tty(&io::stdout()) 
        && crossterm::tty::IsTty::is_tty(&io::stdin());
    
    if !has_tty {
        eprintln!("Error: This application requires a TTY terminal to run.");
        eprintln!("Please run this application in a proper terminal environment.");
        eprintln!("If you're using VS Code, try running it in the integrated terminal.");
        eprintln!("If you're using a CI/automated environment, this application is not suitable for that context.");
        std::process::exit(1);
    }
    
    // Load config and store
    let mut config = AppConfig::load().unwrap_or_default();
    let db_path = config.db_path.clone();
    let store = ConversationStore::new(db_path)?;
    let oaiconfig = AzureOpenAIConfig::from_env().expect("Azure OpenAI configuration is not present. Please set the required environment variables: AZURE_OPENAI_API_KEY (or AZURE_OPENAI_KEY), AZURE_OPENAI_ENDPOINT, AZURE_OPENAI_DEPLOYMENT_ID, and optionally AZURE_OPENAI_API_VERSION and OPENAI_MODEL");
    let oaiclient = AzureOpenAIClient::new(oaiconfig);
    
    // Setup terminal with better error handling
    enable_raw_mode().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| format!("Failed to setup terminal: {}", e))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| format!("Failed to create terminal: {}", e))?;
    
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app, &mut config, &store, &oaiclient).await;
    
    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    
    if let Err(err) = res {
        println!("Error: {}", err);
    }
    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    config: &mut AppConfig,
    store: &ConversationStore,
    oaiclient: &AzureOpenAIClient,
) -> Result<(), Box<dyn std::error::Error>>
where
    B::Error: 'static + std::error::Error,
{
    loop {
        // UI Rendering Domain
        terminal.draw(|f| ui(f, app))?;
        
        // Event Polling Domain
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                // Event Handling Domain - delegate to domain-specific function
                let should_quit = handle_key_event(key.code, app, config, store, oaiclient).await?;
                if should_quit {
                    return Ok(());
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(f.area());

    // Defensive check: only render if there are at least 2 chunks
    if chunks.len() >= 2 {
        // server list
        let items: Vec<ListItem> = app.servers.iter().map(|s| ListItem::new(s.as_str())).collect();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Servers"))
            .highlight_style(Style::default().fg(Color::Yellow))
            .highlight_symbol(">> ");
        f.render_stateful_widget(list, chunks[0], &mut app.state.clone());

        // chat and input
        let inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(chunks[1]);

        let messages: Vec<ListItem> = app
            .current_messages()
            .iter()
            .map(|(srv, m)| {
                let line = if app.current_server() == "All" {
                    format!("[{}] {}: {}", srv, m.role, m.content)
                } else {
                    format!("{}: {}", m.role, m.content)
                };
                ListItem::new(line)
            })
            .collect();
        let history = List::new(messages).block(Block::default().borders(Borders::ALL).title("History"));
        f.render_widget(history, inner[0]);

        let input = Paragraph::new(app.input.as_str())
            .block(Block::default().borders(Borders::ALL).title("Input (Enter to send, q to quit)"));
        f.render_widget(input, inner[1]);
    }
}
