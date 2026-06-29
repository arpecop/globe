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
use crate::ui::AppState;
use crate::crypto::NicknameHasher;
use crate::handshake::HandshakeClient;
use crate::config::Config;

pub async fn run(connect: &str, nickname: Option<String>) -> Result<()> {
    let nickname = nickname.unwrap_or_else(|| "User".to_string());
    let hasher = NicknameHasher::new("device_salt".to_string());
    let user_id = hasher.hash(&nickname, "local_device");

    // Load config and get handshake URL
    let config = Config::load_or_default(&None)?;
    let handshake = HandshakeClient::new(config.bootstrap.handshake_url.clone());

    println!("🌐 Discovering peers...");

    // If connect is localhost, use it directly
    // Otherwise, use handshake to discover peers
    let _server_addr = if connect.contains("localhost") || connect.contains("127.0.0.1") {
        println!("📍 Using local server: {}", connect);
        connect.to_string()
    } else {
        // Discover peers hosting default channel
        println!("🔍 Querying: {}", config.bootstrap.handshake_url);

        match handshake.list_channels().await {
            Ok(channels) => {
                println!("📡 Available channels:");
                for ch in &channels {
                    println!("   #{}: {} peers", ch.name, ch.peer_count);
                }

                // Default to #general
                match handshake.discover("general").await {
                    Ok(peers) => {
                        if peers.len() == 0 {
                            println!("❌ No peers found for #general");
                            println!("💡 Start a server first: globy serve --salt test --port 3000");
                            return Err(anyhow::anyhow!("No peers available"));
                        }

                        // Use first peer
                        let peer = &peers[0];
                        let addr = format!("{}:{}", peer.ip, peer.port);
                        println!("✅ Connecting to peer: {}", addr);
                        addr
                    }
                    Err(e) => {
                        println!("❌ Failed to discover peers: {}", e);
                        println!("💡 Make sure handshake worker is running");
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to fetch channels: {}", e);
                println!("💡 Handshake worker URL: {}", config.bootstrap.handshake_url);
                return Err(e);
            }
        }
    };

    println!("👤 Nickname: {} ({})", nickname, user_id);
    println!("📺 Starting TUI...");

    // Setup terminal
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize app state
    let mut state = AppState::new(nickname, user_id);

    // Add some demo messages
    state.add_message("0xabcd".to_string(), "Welcome to Globy!".to_string());
    state.add_message("0x5a2f".to_string(), "Hello everyone".to_string());

    // Main event loop
    loop {
        terminal.draw(|f| crate::ui::draw_ui(f, &state))?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Enter => state.send_message(),
                    KeyCode::Backspace => state.handle_backspace(),
                    KeyCode::Char(c) => state.handle_input(c),
                    KeyCode::Up => state.select_prev_channel(),
                    KeyCode::Down => state.select_next_channel(),
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;

    println!("✅ Goodbye!");

    Ok(())
}
