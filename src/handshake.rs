use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub ip: String,
    pub port: u16,
}

#[derive(Debug, Serialize)]
pub struct RegisterRequest {
    pub channel: String,
    pub ip: String,
    pub port: u16,
    pub nickname_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterResponse {
    pub status: String,
    pub peer_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct DiscoverResponse {
    pub channel: String,
    pub peer_count: usize,
    pub peers: Vec<Peer>,
}

#[derive(Debug, Deserialize)]
pub struct ChannelsResponse {
    pub channels: Vec<ChannelInfo>,
    pub total: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChannelInfo {
    pub name: String,
    pub peer_count: usize,
}

pub struct HandshakeClient {
    handshake_url: String,
}

impl HandshakeClient {
    pub fn new(handshake_url: String) -> Self {
        Self { handshake_url }
    }

    /// Register as peer hosting a channel
    pub async fn register(
        &self,
        channel: &str,
        ip: &str,
        port: u16,
        nickname_hash: &str,
    ) -> Result<RegisterResponse> {
        let url = format!("{}/register", self.handshake_url);

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&RegisterRequest {
                channel: channel.to_string(),
                ip: ip.to_string(),
                port,
                nickname_hash: nickname_hash.to_string(),
            })
            .send()
            .await?;

        let body = response.json::<RegisterResponse>().await?;
        Ok(body)
    }

    /// Discover peers hosting a channel
    pub async fn discover(&self, channel: &str) -> Result<Vec<Peer>> {
        let url = format!("{}/discover/{}", self.handshake_url, channel);

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        let body = response.json::<DiscoverResponse>().await?;
        Ok(body.peers)
    }

    /// Get list of all active channels
    pub async fn list_channels(&self) -> Result<Vec<ChannelInfo>> {
        let url = format!("{}/channels", self.handshake_url);

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        let body = response.json::<ChannelsResponse>().await?;
        Ok(body.channels)
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/status", self.handshake_url);

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_creation() {
        let peer = Peer {
            ip: "1.2.3.4".to_string(),
            port: 3000,
        };
        assert_eq!(peer.ip, "1.2.3.4");
        assert_eq!(peer.port, 3000);
    }

    #[test]
    fn test_channel_info() {
        let channel = ChannelInfo {
            name: "general".to_string(),
            peer_count: 5,
        };
        assert_eq!(channel.name, "general");
        assert_eq!(channel.peer_count, 5);
    }
}
