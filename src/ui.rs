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
    pub known_peers: Vec<String>,  // List of peer hashes we know
    pub selected_peer: usize,       // Index in known_peers
}

impl AppState {
    pub fn new(nickname: String, peer_hash: String) -> Self {
        // Demo peers
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
            known_peers,
            selected_peer: 0,
        }
    }

    pub fn select_next_peer(&mut self) {
        if self.selected_peer < self.known_peers.len() - 1 {
            self.selected_peer += 1;
        }
    }

    pub fn select_prev_peer(&mut self) {
        if self.selected_peer > 0 {
            self.selected_peer -= 1;
        }
    }

    pub fn confirm_peer(&mut self) {
        self.screen = Screen::Chat;
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

    // Layout: header | messages (scrollable) | padding | input (no box)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),           // Header (compact)
            Constraint::Min(8),              // Messages (scrollable)
            Constraint::Length(3),           // Input area (padding + input, no box)
        ])
        .split(size);

    // Header (compact, no box)
    draw_header(f, state, chunks[0]);

    // Messages (scrollable)
    draw_messages(f, state, chunks[1]);

    // Input (NO BOX, just text floating)
    draw_input(f, state, chunks[2]);
}

fn draw_peer_selection(f: &mut Frame, state: &AppState) {
    let size = f.size();

    let dialog = Paragraph::new(format!(
        "Who do you want to chat with?\n\n{}",
        state
            .known_peers
            .iter()
            .enumerate()
            .map(|(i, peer)| {
                if i == state.selected_peer {
                    format!("▶ {}", peer)
                } else {
                    format!("  {}", peer)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    ))
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::Yellow));

    let centered = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(10),
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
        .join("  ");

    let header_text = format!("{} | {}  [{}]", state.nickname, state.peer_hash, channels);

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan));

    f.render_widget(header, area);
}

fn draw_messages(f: &mut Frame, state: &AppState, area: Rect) {
    // Create message list
    let messages: Vec<ListItem> = state
        .messages
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

    f.render_widget(list, area);
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
