use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap, Gauge},
};
use std::collections::VecDeque;

pub struct Message {
    pub from: String,
    pub content: String,
}

pub enum Screen {
    PeerSelection,
    Chat,
}

pub struct AppState {
    pub nickname: String,
    pub peer_hash: String,
    pub current_channel: String,
    pub channels: Vec<String>,
    pub messages: VecDeque<Message>,
    pub input: String,
    pub input_cursor: usize,
    pub screen: Screen,
    pub peer_input: String,          // Input for peer selection
    pub peer_input_cursor: usize,
    pub known_peers: Vec<String>,    // Recent peers
}

impl AppState {
    pub fn new(nickname: String, peer_hash: String) -> Self {
        // Recent/known peers
        let known_peers = vec![
            "0x7e81fc64".to_string(),
            "0xabcd1234".to_string(),
            "0x5a2f9876".to_string(),
        ];

        Self {
            nickname,
            peer_hash,
            current_channel: "general".to_string(),
            channels: vec!["general".to_string(), "dev".to_string(), "random".to_string()],
            messages: VecDeque::new(),
            input: String::new(),
            input_cursor: 0,
            screen: Screen::PeerSelection,
            peer_input: String::new(),
            peer_input_cursor: 0,
            known_peers,
        }
    }

    // Peer selection input handling
    pub fn peer_input_char(&mut self, c: char) {
        self.peer_input.insert(self.peer_input_cursor, c);
        self.peer_input_cursor += 1;
    }

    pub fn peer_input_backspace(&mut self) {
        if self.peer_input_cursor > 0 {
            self.peer_input.remove(self.peer_input_cursor - 1);
            self.peer_input_cursor -= 1;
        }
    }

    pub fn peer_input_delete(&mut self) {
        if self.peer_input_cursor < self.peer_input.len() {
            self.peer_input.remove(self.peer_input_cursor);
        }
    }

    pub fn peer_input_left(&mut self) {
        if self.peer_input_cursor > 0 {
            self.peer_input_cursor -= 1;
        }
    }

    pub fn peer_input_right(&mut self) {
        if self.peer_input_cursor < self.peer_input.len() {
            self.peer_input_cursor += 1;
        }
    }

    pub fn confirm_peer(&mut self) {
        if !self.peer_input.trim().is_empty() {
            // Entered a peer hash -> connect as GUEST
            self.screen = Screen::Chat;
        }
    }

    pub fn start_as_host(&mut self) {
        // No peer entered -> start as HOST (wait for guests)
        self.screen = Screen::Chat;
        self.peer_input = format!("(hosting as {})", self.peer_hash);
    }

    pub fn back_to_peers(&mut self) {
        self.screen = Screen::PeerSelection;
        self.input.clear();
        self.input_cursor = 0;
    }

    pub fn add_message(&mut self, from: String, content: String) {
        self.messages.push_back(Message { from, content });
        if self.messages.len() > 500 {
            self.messages.pop_front();
        }
    }

    pub fn handle_input(&mut self, c: char) {
        self.input.insert(self.input_cursor, c);
        self.input_cursor += 1;
    }

    pub fn handle_backspace(&mut self) {
        if self.input_cursor > 0 {
            self.input.remove(self.input_cursor - 1);
            self.input_cursor -= 1;
        }
    }

    pub fn handle_delete(&mut self) {
        if self.input_cursor < self.input.len() {
            self.input.remove(self.input_cursor);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.input_cursor > 0 {
            self.input_cursor -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.input_cursor < self.input.len() {
            self.input_cursor += 1;
        }
    }

    pub fn send_message(&mut self) {
        if !self.input.trim().is_empty() {
            self.add_message(self.peer_hash.clone(), self.input.clone());
            self.input.clear();
            self.input_cursor = 0;
        }
    }

    pub fn next_channel(&mut self) {
        let idx = self.channels.iter().position(|c| c == &self.current_channel).unwrap_or(0);
        if idx < self.channels.len() - 1 {
            self.current_channel = self.channels[idx + 1].clone();
        }
    }

    pub fn prev_channel(&mut self) {
        let idx = self.channels.iter().position(|c| c == &self.current_channel).unwrap_or(0);
        if idx > 0 {
            self.current_channel = self.channels[idx - 1].clone();
        }
    }
}

pub fn draw_ui(f: &mut Frame, state: &AppState) {
    match state.screen {
        Screen::PeerSelection => draw_peer_selection(f, state),
        Screen::Chat => draw_chat(f, state),
    }
}

fn draw_chat(f: &mut Frame, state: &AppState) {
    let size = f.size();

    // Calculate input height (message lines + padding)
    let input_lines = state.input.lines().count().max(1);
    let input_height = (input_lines + 1).min(4); // Max 4 lines for input

    // Layout: header | messages | input (calculated)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),                          // Header (minimal)
            Constraint::Min(5),                             // Messages (fill)
            Constraint::Length(input_height as u16 + 1),    // Input (calculated)
        ])
        .split(size);

    // Header (compact, no box)
    draw_header(f, state, chunks[0]);

    // Messages (scrollable)
    draw_messages(f, state, chunks[1]);

    // Input (NO BOX, calculated position)
    draw_input(f, state, chunks[2]);
}

fn draw_peer_selection(f: &mut Frame, state: &AppState) {
    let size = f.size();

    // Create peer input display with cursor
    let cursor_pos = state.peer_input_cursor;
    let display_input = if cursor_pos <= state.peer_input.len() {
        format!("{}│{}", &state.peer_input[0..cursor_pos], &state.peer_input[cursor_pos..])
    } else {
        format!("{}│", state.peer_input)
    };

    // Recent peers list
    let peers_list = state
        .known_peers
        .iter()
        .map(|p| format!("  • {}", p))
        .collect::<Vec<_>>()
        .join("\n");

    let dialog_text = format!(
        "YOUR PEER ID (share with guests):\n\
         ▶ {}\n\n\
         Enter peer hash to connect to:\n\
         {}\n\n\
         Recent peers:\n\
         {}\n\n\
         [Enter] Connect as guest | [Esc] Start as host (wait for guests)",
        state.peer_hash, display_input, peers_list
    );

    let dialog = Paragraph::new(dialog_text)
        .style(Style::default().fg(Color::Cyan));

    let centered = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(18),
            Constraint::Min(0),
        ])
        .split(size);

    f.render_widget(dialog, centered[1]);
}

fn draw_header(f: &mut Frame, state: &AppState, area: Rect) {
    let channels = state
        .channels
        .iter()
        .map(|c| {
            let marker = if c == &state.current_channel { "●" } else { "○" };
            format!("{}#{}", marker, c)
        })
        .collect::<Vec<_>>()
        .join(" ");

    let header_text = format!("{} | {}  {}", state.nickname, state.peer_hash, channels);

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan));

    f.render_widget(header, area);
}

fn draw_messages(f: &mut Frame, state: &AppState, area: Rect) {
    let max_visible = area.height.saturating_sub(1) as usize;

    // Show only the last N messages that fit on screen
    let start_idx = state.messages.len().saturating_sub(max_visible);
    let visible_messages = state.messages.iter().skip(start_idx).collect::<Vec<_>>();
    let msg_count = visible_messages.len();

    // Create message list (from oldest to newest visible)
    let messages: Vec<ListItem> = visible_messages
        .iter()
        .map(|m| {
            let text = if m.from == state.peer_hash {
                // Your message (subtle)
                format!("{} (you): {}", m.from, m.content)
            } else {
                // Other's message
                format!("{}: {}", m.from, m.content)
            };
            ListItem::new(text)
        })
        .collect();

    let list = List::new(messages)
        .style(Style::default());

    // Layout: add padding at top to push messages to bottom
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),                          // Padding (pushes down)
            Constraint::Length(msg_count as u16),        // Messages at bottom
        ])
        .split(area);

    f.render_widget(list, chunks[1]);
}

fn draw_input(f: &mut Frame, state: &AppState, area: Rect) {
    // Split into padding + input line
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),           // Padding line
            Constraint::Min(1),              // Input text (no box)
        ])
        .split(area);

    // Draw cursor with text (NO BOX)
    let cursor_pos = state.input_cursor;
    let display_input = if cursor_pos <= state.input.len() {
        format!("{}│{}", &state.input[0..cursor_pos], &state.input[cursor_pos..])
    } else {
        format!("{}│", state.input)
    };

    let input_widget = Paragraph::new(display_input)
        .style(Style::default().fg(Color::Green));

    f.render_widget(input_widget, chunks[1]);
}
