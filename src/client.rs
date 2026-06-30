use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io;
use std::time::Instant;
use crate::ui::AppState;
use crate::ssh_key::{SshIdentity, NicknameDatabase};

struct RelayClient {
    relay_url: String,
    peer_hash: String,
}

impl RelayClient {
    fn new(relay_url: String, peer_hash: String) -> Self {
        Self { relay_url, peer_hash }
    }

    async fn send_message(&self, content: String) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!("http://{}/send-message", self.relay_url);

        let payload = serde_json::json!({
            "content": content,
            "from_hash": self.peer_hash,
        });

        let _ = client.post(&url)
            .json(&payload)
            .send()
            .await;

        Ok(())
    }

    async fn fetch_messages(&self) -> Result<Vec<(String, String)>> {
        let client = reqwest::Client::new();
        let url = format!("http://{}/messages", self.relay_url);

        match client.get(&url).send().await {
            Ok(resp) => {
                if let Ok(text) = resp.text().await {
                    // Parse simple format: "0xhash: message\n0xhash: message\n"
                    let messages = text.lines()
                        .filter_map(|line| {
                            if let Some(pos) = line.find(": ") {
                                let from = line[..pos].to_string();
                                let content = line[pos + 2..].to_string();
                                Some((from, content))
                            } else {
                                None
                            }
                        })
                        .collect();
                    Ok(messages)
                } else {
                    Ok(Vec::new())
                }
            }
            Err(_) => Ok(Vec::new()),
        }
    }
}

pub async fn run(connect: &str, nickname: Option<String>) -> Result<()> {
    let nickname = nickname.unwrap_or_else(|| "User".to_string());

    // Load SSH identity (from ~/.ssh/id_ed25519) or generate temp ID
    let peer_hash = match SshIdentity::new() {
        Ok(identity) => {
            println!("🔑 SSH Key loaded");
            identity.get_peer_hash()?
        }
        Err(_) => {
            // No SSH key, generate temporary ID
            use sha2::{Sha256, Digest};
            use hex::encode;
            let input = format!("temp_{}", uuid::Uuid::new_v4());
            let mut hasher = Sha256::new();
            hasher.update(input.as_bytes());
            let result = hasher.finalize();
            let hex = encode(result);
            format!("0x{}", &hex[0..8])
        }
    };

    let db = NicknameDatabase::new()?;
    db.set(&peer_hash, &nickname)?;

    println!("✅ Connected!");
    println!("👤 Your name: {}", nickname);
    println!("📍 Relay: {}", connect);
    println!("📺 Starting Globy...\n");

    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new(nickname, peer_hash.clone());
    state.start_as_host(); // Auto-enter chat mode

    let relay = RelayClient::new(connect.to_string(), peer_hash.clone());
    let mut last_fetch = Instant::now();
    let mut seen_messages = std::collections::HashSet::new();

    loop {
        // Fetch new messages every 500ms
        if last_fetch.elapsed().as_millis() > 500 {
            if let Ok(messages) = relay.fetch_messages().await {
                for (from, content) in messages {
                    let key = format!("{}:{}", from, content);
                    if !seen_messages.contains(&key) {
                        seen_messages.insert(key);
                        state.add_message(from, content);
                    }
                }
            }
            last_fetch = Instant::now();
        }

        terminal.draw(|f| crate::ui::draw_ui(f, &state))?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Enter => {
                        if !state.input.is_empty() {
                            let msg = state.input.clone();
                            state.add_message(peer_hash.clone(), msg.clone());
                            let _ = relay.send_message(msg).await;
                            state.input.clear();
                            state.input_cursor = 0;
                        }
                    }
                    KeyCode::Backspace => state.handle_backspace(),
                    KeyCode::Delete => state.handle_delete(),
                    KeyCode::Left => state.move_cursor_left(),
                    KeyCode::Right => state.move_cursor_right(),
                    KeyCode::Char(c) => state.handle_input(c),
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    println!("✅ Goodbye!");

    Ok(())
}
