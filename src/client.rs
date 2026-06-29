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
use crate::ssh_key::{SshIdentity, NicknameDatabase};

pub async fn run(connect: &str, nickname: Option<String>) -> Result<()> {
    let nickname = nickname.unwrap_or_else(|| "User".to_string());

    // Load SSH identity (from ~/.ssh/id_ed25519)
    let identity = SshIdentity::new()?;
    let peer_hash = identity.get_peer_hash()?;

    // Load nickname database (stores local mapping)
    let db = NicknameDatabase::new()?;
    db.set(&peer_hash, &nickname)?;

    println!("👤 Nickname: {} ({})", nickname, peer_hash);
    println!("🔑 SSH Key loaded");
    println!("🌐 Connecting to: {}", connect);
    println!("📺 Starting TUI...");

    // Setup terminal
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize app state
    let mut state = AppState::new(nickname, peer_hash);

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
