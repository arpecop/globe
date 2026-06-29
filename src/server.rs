use anyhow::Result;
use crate::config::Config;
use dashmap::DashMap;
use std::sync::Arc;
use crate::protocol::ChannelState;

pub struct Server {
    config: Config,
    channels: Arc<DashMap<String, ChannelState>>,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            channels: Arc::new(DashMap::new()),
        }
    }

    pub async fn run(&self, port: u16, mode: &str) -> Result<()> {
        println!("📡 Server starting on port {}", port);
        println!("🔐 Mode: {}", mode);

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
    let server = Server::new(config);
    server.run(port, mode).await
}
