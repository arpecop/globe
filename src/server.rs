use anyhow::Result;
use crate::config::Config;
use crate::ssh_key::SshIdentity;
use crate::handshake::HeartbeatClient;
use dashmap::DashMap;
use std::sync::Arc;
use crate::protocol::ChannelState;

pub struct Server {
    config: Config,
    channels: Arc<DashMap<String, ChannelState>>,
    peer_hash: String,
    heartbeat_client: HeartbeatClient,
}

impl Server {
    pub fn new(config: Config, peer_hash: String, heartbeat_client: HeartbeatClient) -> Self {
        Self {
            config,
            channels: Arc::new(DashMap::new()),
            peer_hash,
            heartbeat_client,
        }
    }

    pub async fn run(&self, port: u16, mode: &str) -> Result<()> {
        println!("📡 Server starting on port {}", port);
        println!("🔐 Peer Hash: {}", self.peer_hash);
        println!("📍 Mode: {}", mode);

        // Start heartbeat loop
        let heartbeat_client = self.heartbeat_client.clone();
        let peer_hash = self.peer_hash.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = heartbeat_client.heartbeat(&peer_hash).await {
                    tracing::error!("Heartbeat failed: {}", e);
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });

        match mode {
            "api" => self.run_api(port).await?,
            "tui" => self.run_tui().await?,
            "both" => {
                tokio::select! {
                    _ = self.run_api(port) => {},
                    _ = self.run_tui() => {},
                }
            }
            _ => println!("Unknown mode: {}", mode),
        }

        Ok(())
    }

    async fn run_api(&self, port: u16) -> Result<()> {
        println!("🌐 API server listening on port {}", port);
        println!("✅ Online and accepting connections");
        // TODO: Implement REST API with axum
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    async fn run_tui(&self) -> Result<()> {
        println!("📺 TUI mode not yet implemented");
        // TODO: Implement terminal UI
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}

pub async fn run(config: Config, port: u16, mode: &str) -> Result<()> {
    // Get SSH identity
    let identity = SshIdentity::new()?;
    let peer_hash = identity.get_peer_hash()?;

    println!("🔑 SSH Key: {}", identity.get_public_key()?.lines().next().unwrap_or("...")[0..40].to_string());
    println!("🆔 Your peer hash: {}", peer_hash);

    let heartbeat_client = HeartbeatClient::new(config.bootstrap.handshake_url.clone());

    let server = Server::new(config, peer_hash, heartbeat_client);
    server.run(port, mode).await
}
