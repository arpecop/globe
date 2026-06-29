use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap, Gauge},
};
use std::collections::VecDeque;

pub struct Message {
    pub from: String,
    pub content: String,
}

pub struct AppState {
    pub nickname: String,
    pub peer_hash: String,
    pub current_channel: String,
    pub channels: Vec<String>,
    pub messages: VecDeque<Message>,
    pub input: String,
    pub input_cursor: usize,
}

impl AppState {
    pub fn new(nickname: String, peer_hash: String) -> Self {
        Self {
            nickname,
            peer_hash,
            current_channel: "general".to_string(),
            channels: vec!["general".to_string(), "dev".to_string(), "random".to_string()],
            messages: VecDeque::new(),
            input: String::new(),
            input_cursor: 0,
        }
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
    let size = f.size();

    // Layout: header | messages (scrollable) | input (fixed)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),           // Header
            Constraint::Min(5),              // Messages (scrollable)
            Constraint::Length(4),           // Input (fixed at bottom)
        ])
        .split(size);

    // Header
    draw_header(f, state, chunks[0]);

    // Messages (scrollable)
    draw_messages(f, state, chunks[1]);

    // Input (fixed at bottom, like Claude Code)
    draw_input(f, state, chunks[2]);
}

fn draw_header(f: &mut Frame, state: &AppState, area: Rect) {
    let channels = state
        .channels
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let marker = if c == &state.current_channel { "●" } else { "○" };
            format!(" {} #{}", marker, c)
        })
        .collect::<Vec<_>>()
        .join("  ");

    let header = Paragraph::new(format!(
        " {} | {} \n {}",
        state.nickname, state.peer_hash, channels
    ))
    .block(Block::default().borders(Borders::BOTTOM))
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
                // Your message (highlighted)
                format!("  {} (you): {}", m.from, m.content)
            } else {
                // Other's message
                format!("  {}: {}", m.from, m.content)
            };
            ListItem::new(text)
        })
        .collect();

    let list = List::new(messages)
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default());

    f.render_widget(list, area);
}

fn draw_input(f: &mut Frame, state: &AppState, area: Rect) {
    let input_lines = state.input.lines().count().max(1);
    let input_height = input_lines.min(5); // Max 5 lines

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(input_height as u16)])
        .split(area);

    let input_area = chunks[1];

    // Draw cursor position
    let cursor_pos = state.input_cursor;
    let display_input = if cursor_pos <= state.input.len() {
        format!("{}│{}", &state.input[0..cursor_pos], &state.input[cursor_pos..])
    } else {
        format!("{}│", state.input)
    };

    let input_widget = Paragraph::new(display_input)
        .block(
            Block::default()
                .title(" Message ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Green))
        )
        .wrap(Wrap { trim: false });

    f.render_widget(input_widget, input_area);
}
