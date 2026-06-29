use anyhow::Result;
use crate::config::Config;
use crate::ssh_key::SshIdentity;
use crate::handshake::HeartbeatClient;

pub struct Server {
    peer_hash: String,
    nickname: String,
    port: u16,
    heartbeat_client: HeartbeatClient,
}

impl Server {
    pub fn new(peer_hash: String, nickname: String, port: u16, heartbeat_client: HeartbeatClient) -> Self {
        Self {
            peer_hash,
            nickname,
            port,
            heartbeat_client,
        }
    }

    pub async fn run(&self) -> Result<()> {
        println!("📡 Handshake Server");
        println!("🆔 Peer: {} ({})", self.peer_hash, self.nickname);
        println!("📍 Port: {}", self.port);
        println!("⚡ Mode: Handshake only (no relaying)");

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

        println!("✅ Online");
        println!("📡 Waiting for handshake requests...");

        // TODO: Implement TCP/WebSocket server for handshakes
        // For now, just keep alive
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}

pub async fn run(config: Config, port: u16, nickname: String) -> Result<()> {
    // Get SSH identity
    let identity = SshIdentity::new()?;
    let peer_hash = identity.get_peer_hash()?;

    let heartbeat_client = HeartbeatClient::new(config.bootstrap.handshake_url.clone());

    let server = Server::new(peer_hash, nickname, port, heartbeat_client);
    server.run().await
}
