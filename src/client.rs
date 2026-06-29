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

    // Load nickname database (stores local mapping)
    let db = NicknameDatabase::new()?;
    db.set(&peer_hash, &nickname)?;

    println!("✅ Ready!");
    println!("👤 Your name: {}", nickname);
    println!("📺 Starting Globy...\n");

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
                match state.screen {
                    crate::ui::Screen::PeerSelection => {
                        match key.code {
                            KeyCode::Esc => {
                                // Esc = start as HOST (wait for guests)
                                state.start_as_host();
                            }
                            KeyCode::Enter => {
                                // Enter = connect as GUEST (if peer entered)
                                state.confirm_peer();
                            }
                            KeyCode::Backspace => state.peer_input_backspace(),
                            KeyCode::Delete => state.peer_input_delete(),
                            KeyCode::Left => state.peer_input_left(),
                            KeyCode::Right => state.peer_input_right(),
                            KeyCode::Char(c) => state.peer_input_char(c),
                            _ => {}
                        }
                    }
                    crate::ui::Screen::Chat => {
                        match key.code {
                            KeyCode::Esc => state.back_to_peers(),
                            KeyCode::Enter => state.send_message(),
                            KeyCode::Backspace => state.handle_backspace(),
                            KeyCode::Delete => state.handle_delete(),
                            KeyCode::Left => state.move_cursor_left(),
                            KeyCode::Right => state.move_cursor_right(),
                            KeyCode::Up if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => state.next_channel(),
                            KeyCode::Down if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => state.prev_channel(),
                            KeyCode::Char(c) => state.handle_input(c),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;

    println!("✅ Goodbye!");

    Ok(())
}
