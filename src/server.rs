use anyhow::Result;
use crate::config::Config;
use crate::ssh_key::SshIdentity;
use crate::handshake::HeartbeatClient;
use crate::api::{SendMessageResponse, EncryptedMessageRequest};
use crate::crypto::MessageEncryption;
use crate::ui::{AppState};
use axum::{
    extract::Json,
    routing::post,
    Router,
    http::StatusCode,
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

#[derive(Clone, Debug)]
pub struct Message {
    pub id: String,
    pub from_hash: String,
    pub to_hash: String,
    pub nickname: String,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct MessageQueue {
    messages: Arc<Mutex<VecDeque<Message>>>,
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn add(&self, message: Message) {
        let mut msgs = self.messages.lock().await;
        msgs.push_back(message);
        if msgs.len() > 1000 {
            msgs.pop_front();
        }
    }

    pub async fn get_all(&self) -> Vec<Message> {
        let msgs = self.messages.lock().await;
        msgs.iter().cloned().collect()
    }

    /// Get messages for a specific recipient (for relay polling)
    pub async fn get_for_recipient(&self, to_hash: &str) -> Vec<Message> {
        let msgs = self.messages.lock().await;
        msgs.iter()
            .filter(|m| m.to_hash == to_hash)
            .cloned()
            .collect()
    }

    /// Delete message after delivery
    pub async fn delete(&self, message_id: &str) {
        let mut msgs = self.messages.lock().await;
        msgs.retain(|m| m.id != message_id);
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

            match std::net::TcpListener::bind(format!("0.0.0.0:{}", port)) {
                Ok(std_listener) => {
                    std_listener.set_nonblocking(true).ok();

                    let tokio_listener = tokio::net::TcpListener::from_std(std_listener).ok();
                    if let Some(listener) = tokio_listener {
                        let _ = axum::serve(listener, app).await;
                    }
                }
                Err(_) => {}
            }
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        println!("✅ API Ready - Listening on 0.0.0.0:{}\n", self.port);

        // Try to run TUI, but keep server running if it fails
        if let Err(e) = self.run_tui().await {
            println!("[WARN] TUI failed: {} - server will continue running", e);
            println!("[INFO] API endpoint available at http://localhost:{}/send-message", self.port);

            // Keep the server running indefinitely
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        }

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
                state.add_message(msg.from_hash.clone(), format!("{}: {}", msg.nickname, msg.content));
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
    Json(payload): Json<EncryptedMessageRequest>,
) -> Result<Json<SendMessageResponse>, (StatusCode, String)> {
    // Step 1: Load our X25519 private key
    let identity = SshIdentity::new()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load identity: {}", e)))?;

    let our_x25519_private = identity.get_x25519_private_key()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load X25519 key: {}", e)))?;

    // Step 2: Verify SSH signature
    let message_to_verify = format!("{}||{}||{}", payload.to_hash, payload.ephemeral_pubkey, payload.ciphertext);
    let signature_valid = identity.verify_ssh_signature(&message_to_verify, &payload.signature, &payload.ssh_pubkey)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Signature verification error: {}", e)))?;

    if !signature_valid {
        return Err((StatusCode::UNAUTHORIZED, "Invalid SSH signature".to_string()));
    }

    // Step 3: Derive shared secret
    let shared_secret = MessageEncryption::derive_shared_secret(
        &our_x25519_private,
        &payload.ephemeral_pubkey,
    ).map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to derive shared secret: {}", e)))?;

    // Step 4: Decrypt the message
    let plaintext = MessageEncryption::decrypt(
        &payload.ciphertext,
        &payload.nonce,
        &payload.tag,
        &shared_secret,
        &format!("{}|{}", payload.from_hash, payload.to_hash),
    ).map_err(|e| (StatusCode::BAD_REQUEST, format!("Decryption failed: {}", e)))?;

    // Step 5: Parse the decrypted message
    let decrypted_msg: serde_json::Value = serde_json::from_str(&plaintext)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid message format: {}", e)))?;

    let nickname = decrypted_msg.get("nickname")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let content = decrypted_msg.get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("(empty message)")
        .to_string();

    let timestamp = decrypted_msg.get("timestamp")
        .and_then(|v| v.as_u64())
        .unwrap_or_else(|| chrono::Utc::now().timestamp() as u64);

    // Step 6: Add to queue
    let message_id = uuid::Uuid::new_v4().to_string();
    let message = Message {
        id: message_id.clone(),
        from_hash: payload.from_hash.clone(),
        to_hash: payload.to_hash.clone(),
        nickname,
        content,
        timestamp,
    };
    queue.add(message).await;

    Ok(Json(SendMessageResponse::success(message_id)))
}


pub async fn run(config: Config, port: u16, nickname: String) -> Result<()> {
    let identity = SshIdentity::new()?;
    let peer_hash = identity.get_peer_hash()?;

    let heartbeat_client = HeartbeatClient::new(config.bootstrap.handshake_url.clone());

    let server = Server::new(peer_hash, nickname, port, heartbeat_client);
    server.run().await
}
