use anyhow::Result;
use crate::config::Config;
use crate::ssh_key::SshIdentity;
use crate::handshake::HeartbeatClient;
use crate::api::{SendMessageRequest, SendMessageResponse};
use crate::ui::{AppState, Screen};
use axum::{
    extract::Json,
    routing::post,
    Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::VecDeque;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io;

#[derive(Clone)]
pub struct MessageQueue {
    messages: Arc<Mutex<VecDeque<String>>>,
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn add(&self, message: String) {
        let mut msgs = self.messages.lock().await;
        msgs.push_back(message);
        if msgs.len() > 100 {
            msgs.pop_front();
        }
    }

    pub async fn get_all(&self) -> Vec<String> {
        let msgs = self.messages.lock().await;
        msgs.iter().cloned().collect()
    }
}

pub struct Server {
    peer_hash: String,
    nickname: String,
    port: u16,
    heartbeat_client: HeartbeatClient,
    message_queue: MessageQueue,
}

impl Server {
    pub fn new(peer_hash: String, nickname: String, port: u16, heartbeat_client: HeartbeatClient) -> Self {
        Self {
            peer_hash,
            nickname,
            port,
            heartbeat_client,
            message_queue: MessageQueue::new(),
        }
    }

    pub async fn run(&self) -> Result<()> {
        println!("📡 Server with Chat UI");
        println!("🆔 Your ID: {} ({})", self.peer_hash, self.nickname);
        println!("📍 API Port: {}", self.port);
        println!("📨 Send messages: curl -X POST http://localhost:{}/send-message -H 'Content-Type: application/json' -d '{{\"to_hash\":\"{}\",\"content\":\"msg\"}}'", self.port, self.peer_hash);

        // Start heartbeat
        let heartbeat_client = self.heartbeat_client.clone();
        let peer_hash = self.peer_hash.clone();
        tokio::spawn(async move {
            loop {
                let _ = heartbeat_client.heartbeat(&peer_hash).await;
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });

        // Start HTTP API server in background
        let message_queue = self.message_queue.clone();
        let port = self.port;
        tokio::spawn(async move {
            let app = Router::new()
                .route("/send-message", post(handle_send_message))
                .with_state(message_queue);

            let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.ok();
            if let Some(l) = listener {
                let _ = axum::serve(l, app).await;
            }
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        println!("✅ API Ready - Now showing chat...\n");

        // Run TUI in main task
        self.run_tui().await?;

        Ok(())
    }

    async fn run_tui(&self) -> Result<()> {
        enable_raw_mode()?;
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut state = AppState::new(self.nickname.clone(), self.peer_hash.clone());

        // Auto-start as HOST (no peer selection)
        state.start_as_host();

        loop {
            // Get latest messages from queue
            let all_messages = self.message_queue.get_all().await;
            state.messages.clear();
            for msg in all_messages {
                // Parse "0xabcd: hello" format
                if let Some(pos) = msg.find(": ") {
                    let from = msg[..pos].to_string();
                    let content = msg[pos + 2..].to_string();
                    state.add_message(from, content);
                } else {
                    state.add_message("0x????".to_string(), msg);
                }
            }

            terminal.draw(|f| crate::ui::draw_ui(f, &state))?;

            if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char(c) => state.handle_input(c),
                        KeyCode::Enter => state.send_message(),
                        KeyCode::Backspace => state.handle_backspace(),
                        _ => {}
                    }
                }
            }
        }

        disable_raw_mode()?;
        Ok(())
    }
}

async fn handle_send_message(
    axum::extract::State(queue): axum::extract::State<MessageQueue>,
    Json(payload): Json<SendMessageRequest>,
) -> Json<SendMessageResponse> {
    let message = format!("{}: {}",
        payload.from_hash.unwrap_or_else(|| "guest".to_string()),
        payload.content
    );
    queue.add(message).await;

    Json(SendMessageResponse::success("msg_123".to_string()))
}

pub async fn run(config: Config, port: u16, nickname: String) -> Result<()> {
    let identity = SshIdentity::new()?;
    let peer_hash = identity.get_peer_hash()?;

    let heartbeat_client = HeartbeatClient::new(config.bootstrap.handshake_url.clone());

    let server = Server::new(peer_hash, nickname, port, heartbeat_client);
    server.run().await
}
