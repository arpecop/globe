use anyhow::Result;
use crate::config::Config;
use crate::ssh_key::SshIdentity;
use crate::handshake::HeartbeatClient;
use crate::api::{SendMessageRequest, SendMessageResponse};
use axum::{
    extract::Json,
    routing::post,
    Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::VecDeque;

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
        // Keep last 100 messages
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
        println!("📡 API Server");
        println!("🆔 Peer: {} ({})", self.peer_hash, self.nickname);
        println!("📍 Port: {}", self.port);
        println!("📨 API: http://0.0.0.0:{}/send-message", self.port);

        // Start heartbeat loop
        let heartbeat_client = self.heartbeat_client.clone();
        let peer_hash = self.peer_hash.clone();
        tokio::spawn(async move {
            loop {
                match heartbeat_client.heartbeat(&peer_hash).await {
                    Ok(_) => tracing::debug!("Heartbeat sent"),
                    Err(e) => tracing::error!("Heartbeat failed: {}", e),
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });

        // Build router
        let message_queue = self.message_queue.clone();
        let app = Router::new()
            .route("/send-message", post(handle_send_message))
            .with_state(message_queue);

        // Start HTTP server
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        println!("✅ Online - API ready");
        println!("📝 Example: curl -X POST http://localhost:{}/send-message -H 'Content-Type: application/json' -d '{{\"to_hash\":\"0x7e81fc64\",\"content\":\"hello\"}}'", self.port);

        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn handle_send_message(
    axum::extract::State(queue): axum::extract::State<MessageQueue>,
    Json(payload): Json<SendMessageRequest>,
) -> Json<SendMessageResponse> {
    let message = format!("{}: {}", payload.from_hash.unwrap_or_else(|| "0x????".to_string()), payload.content);
    queue.add(message).await;

    Json(SendMessageResponse::success("msg_123".to_string()))
}

pub async fn run(config: Config, port: u16, nickname: String) -> Result<()> {
    // Get SSH identity
    let identity = SshIdentity::new()?;
    let peer_hash = identity.get_peer_hash()?;

    let heartbeat_client = HeartbeatClient::new(config.bootstrap.handshake_url.clone());

    let server = Server::new(peer_hash, nickname, port, heartbeat_client);
    server.run().await
}
