use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from: String,           // User ID (hash)
    pub channel: String,        // Channel name
    pub content: String,
    pub timestamp: u64,
    pub signature: String,      // ed25519 signature
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,           // e.g., "general"
    pub description: Option<String>,
    pub peer_count: usize,
    pub created_at: u64,
    pub hosted_by_you: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,             // Hashed nickname
    pub nickname: String,       // Original nickname (local only)
    pub device_id: String,
    pub public_key: Vec<u8>,    // ed25519 public key
    pub joined_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub address: String,        // IP:port
    pub channels: Vec<String>,
    pub last_seen: u64,
    pub hosting_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    MessageReceived(Message),
    UserJoined { user_id: String, channel: String },
    UserLeft { user_id: String, channel: String },
    PeerConnected(Peer),
    PeerDisconnected(String),
    ChannelCreated(Channel),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub msg_type: String,       // "message", "join", "leave", "ping", etc
    pub data: serde_json::Value,
    pub timestamp: u64,
}

impl ProtocolMessage {
    pub fn new_message(from: &str, channel: &str, content: &str) -> Self {
        Self {
            msg_type: "message".to_string(),
            data: serde_json::json!({
                "from": from,
                "channel": channel,
                "content": content,
            }),
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }

    pub fn new_join(user_id: &str, channel: &str) -> Self {
        Self {
            msg_type: "join".to_string(),
            data: serde_json::json!({
                "user_id": user_id,
                "channel": channel,
            }),
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }

    pub fn new_leave(user_id: &str, channel: &str) -> Self {
        Self {
            msg_type: "leave".to_string(),
            data: serde_json::json!({
                "user_id": user_id,
                "channel": channel,
            }),
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }

    pub fn new_ping() -> Self {
        Self {
            msg_type: "ping".to_string(),
            data: serde_json::json!({}),
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChannelState {
    pub channel: Channel,
    pub messages: Vec<Message>,     // Ephemeral, cleared on server restart
    pub users: HashMap<String, User>,
}

impl ChannelState {
    pub fn new(name: String) -> Self {
        Self {
            channel: Channel {
                id: name.clone(),
                name,
                description: None,
                peer_count: 0,
                created_at: chrono::Utc::now().timestamp() as u64,
                hosted_by_you: false,
            },
            messages: Vec::new(),
            users: HashMap::new(),
        }
    }

    pub fn add_message(&mut self, msg: Message) {
        self.messages.push(msg);
        // Keep only last 100 messages in memory
        if self.messages.len() > 100 {
            self.messages.remove(0);
        }
    }

    pub fn join_user(&mut self, user: User) {
        self.users.insert(user.id.clone(), user);
        self.channel.peer_count = self.users.len();
    }

    pub fn leave_user(&mut self, user_id: &str) {
        self.users.remove(user_id);
        self.channel.peer_count = self.users.len();
    }
}
