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
use openai::{send_chat, Message};

struct App {
    servers: Vec<String>,
    state: ListState,
    input: String,
    messages: HashMap<String, Vec<Message>>, // server -> messages
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
        Self { servers, state, input: String::new(), messages }
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
    let mut list = vec!["All".to_string()];
    if let Ok(read) = fs::read_dir("mcp_servers") {
        for entry in read.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let name = entry.file_name().to_string_lossy().into_owned();
                list.push(name);
            }
        }
    }
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app).await;
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
) -> Result<(), Box<dyn std::error::Error>>
where
    B::Error: 'static + std::error::Error,
{
    loop {
        terminal.draw(|f| ui(f, app))?;
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Up => {
                        if let Some(selected) = app.state.selected() {
                            let new = selected.saturating_sub(1);
                            app.state.select(Some(new));
                        }
                    }
                    KeyCode::Down => {
                        if let Some(selected) = app.state.selected() {
                            let new = (selected + 1).min(app.servers.len().saturating_sub(1));
                            app.state.select(Some(new));
                        }
                    }
                    KeyCode::Enter => {
                        let server = app.current_server();
                        if server != "All" && !app.input.is_empty() {
                            let msgs = app.messages.entry(server.clone()).or_default();
                            msgs.push(Message { role: "user".into(), content: app.input.clone() });
                            let reply = match send_chat(msgs).await {
                                Ok(r) => r,
                                Err(e) => Message { role: "error".into(), content: format!("{}", e) },
                            };
                            msgs.push(reply);
                            let _ = save_history(&server, msgs);
                            app.input.clear();
                        }
                    }
                    KeyCode::Backspace => { app.input.pop(); }
                    KeyCode::Char(c) => { app.input.push(c); }
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(f.size());

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
