use anyhow::Result;
use russh::server::{Auth, Session};
use russh::{server::*, ChannelId};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::ssh_key::SshIdentity;
use crate::server::MessageQueue;

/// SSH Server for Globy - users connect via SSH to access encrypted chat
pub struct GlobySSHServer {
    peer_hash: String,
    message_queue: Arc<Mutex<MessageQueue>>,
}

impl GlobySSHServer {
    pub fn new(peer_hash: String, message_queue: Arc<Mutex<MessageQueue>>) -> Self {
        Self {
            peer_hash,
            message_queue,
        }
    }

    /// Start SSH server listening on port 2222
    pub async fn run(&self, port: u16) -> Result<()> {
        let config = russh::server::Config::default();
        let config = Arc::new(config);

        // Create host key (or load existing)
        let identity = SshIdentity::new()?;
        let ssh_pubkey = identity.get_public_key()?;

        println!("🔑 SSH Server starting on 0.0.0.0:{}", port);
        println!("📡 Users can connect: ssh <username>@<your-ip>:{}", port);
        println!("🆔 This peer: {}", self.peer_hash);

        // TODO: Implement actual russh::Server trait and bind to socket
        // For now, this is the structure

        Ok(())
    }
}

/// SSH session handler - manages individual SSH connections
#[derive(Clone)]
pub struct SessionHandler {
    peer_hash: String,
    message_queue: Arc<Mutex<MessageQueue>>,
    username: String,
}

impl SessionHandler {
    pub fn new(
        peer_hash: String,
        message_queue: Arc<Mutex<MessageQueue>>,
        username: String,
    ) -> Self {
        Self {
            peer_hash,
            message_queue,
            username,
        }
    }

    /// Handle incoming SSH shell request - launch TUI chat
    pub async fn handle_shell(&self) -> Result<()> {
        println!("✅ {} logged in", self.username);
        // TODO: Spawn TUI in this session
        Ok(())
    }
}

// Example: How to use this
// In main.rs, add:
//   let ssh_server = GlobySSHServer::new(peer_hash, Arc::new(Mutex::new(message_queue)));
//   ssh_server.run(2222).await?;
