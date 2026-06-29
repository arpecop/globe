use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use std::collections::VecDeque;

pub struct Message {
    pub from: String,
    pub content: String,
    pub timestamp: u64,
}

pub struct AppState {
    pub nickname: String,
    pub user_id: String,
    pub current_channel: String,
    pub channels: Vec<String>,
    pub messages: VecDeque<Message>,
    pub input: String,
    pub channel_list_state: usize,
    pub scroll_offset: usize,
}

impl AppState {
    pub fn new(nickname: String, user_id: String) -> Self {
        Self {
            nickname,
            user_id,
            current_channel: "general".to_string(),
            channels: vec!["general".to_string(), "dev".to_string(), "random".to_string()],
            messages: VecDeque::new(),
            input: String::new(),
            channel_list_state: 0,
            scroll_offset: 0,
        }
    }

    pub fn add_message(&mut self, from: String, content: String) {
        self.messages.push_back(Message {
            from,
            content,
            timestamp: chrono::Utc::now().timestamp() as u64,
        });
        if self.messages.len() > 100 {
            self.messages.pop_front();
        }
    }

    pub fn handle_input(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn handle_backspace(&mut self) {
        self.input.pop();
    }

    pub fn send_message(&mut self) {
        if !self.input.trim().is_empty() {
            self.add_message(self.user_id.clone(), self.input.clone());
            self.input.clear();
        }
    }

    pub fn select_next_channel(&mut self) {
        if self.channel_list_state < self.channels.len() - 1 {
            self.channel_list_state += 1;
            self.current_channel = self.channels[self.channel_list_state].clone();
        }
    }

    pub fn select_prev_channel(&mut self) {
        if self.channel_list_state > 0 {
            self.channel_list_state -= 1;
            self.current_channel = self.channels[self.channel_list_state].clone();
        }
    }
}

pub fn draw_ui(f: &mut Frame, state: &AppState) {
    let size = f.size();

    // Main layout: channels (left) | messages (center) | users (right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(40), Constraint::Length(15)])
        .split(size);

    // Left panel: Channels
    draw_channels(f, state, chunks[0]);

    // Center panel: Messages
    draw_messages(f, state, chunks[1]);

    // Right panel: Users & Info
    draw_info(f, state, chunks[2]);
}

fn draw_channels(f: &mut Frame, state: &AppState, area: Rect) {
    let channels = state
        .channels
        .iter()
        .map(|c| {
            let prefix = if c == &state.current_channel { "▶" } else { " " };
            ListItem::new(format!("{} #{}", prefix, c))
        })
        .collect::<Vec<_>>();

    let list = List::new(channels)
        .block(Block::default().title("Channels").borders(Borders::ALL))
        .style(Style::default());

    f.render_widget(list, area);
}

fn draw_messages(f: &mut Frame, state: &AppState, area: Rect) {
    let message_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(area);

    // Messages list
    let messages = state
        .messages
        .iter()
        .map(|m| {
            let text = format!("{}: {}", m.from, m.content);
            ListItem::new(text)
        })
        .collect::<Vec<_>>();

    let list = List::new(messages)
        .block(
            Block::default()
                .title(format!("#{}", state.current_channel))
                .borders(Borders::ALL),
        )
        .style(Style::default());

    f.render_widget(list, message_area[0]);

    // Input box
    let input_text = Paragraph::new(state.input.clone())
        .block(Block::default().title("You").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    f.render_widget(input_text, message_area[1]);
}

fn draw_info(f: &mut Frame, state: &AppState, area: Rect) {
    let info_text = format!(
        "User: {}\n{}\nPeers: 42\nHosting: 18",
        state.user_id, state.nickname
    );

    let widget = Paragraph::new(info_text)
        .block(Block::default().title("Info").borders(Borders::ALL))
        .alignment(Alignment::Left);

    f.render_widget(widget, area);
}
