use anyhow::Result;
use serde::Deserialize;

/// Ultra-minimal heartbeat client
/// Worker stores ZERO metadata, only confirms "someone is online"
#[derive(Clone)]
pub struct HeartbeatClient {
    worker_url: String,
}

#[derive(Debug, Deserialize)]
pub struct HeartbeatResponse {
    pub ok: bool,
}

#[derive(Debug, Deserialize)]
pub struct StatusResponse {
    pub online: bool,
}

#[derive(Debug, Deserialize)]
pub struct PingResponse {
    pub status: String,
    pub peers_online: usize,
}

impl HeartbeatClient {
    pub fn new(worker_url: String) -> Self {
        Self { worker_url }
    }

    /// Send heartbeat (I'm alive!)
    /// This is the ONLY thing sent to worker.
    /// NO IP, NO nickname, NO metadata.
    pub async fn heartbeat(&self, peer_hash: &str) -> Result<bool> {
        let url = format!("{}/heartbeat/{}", self.worker_url, peer_hash);

        let client = reqwest::Client::new();
        let response = client.post(&url).send().await?;

        let body = response.json::<HeartbeatResponse>().await?;
        Ok(body.ok)
    }

    /// Check if anyone is online in a channel
    /// Returns: true/false (no peer info)
    pub async fn is_channel_online(&self, channel: &str) -> Result<bool> {
        let url = format!("{}/status/{}", self.worker_url, channel);

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        let body = response.json::<StatusResponse>().await?;
        Ok(body.online)
    }

    /// Health check
    pub async fn ping(&self) -> Result<usize> {
        let url = format!("{}/ping", self.worker_url);

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        let body = response.json::<PingResponse>().await?;
        Ok(body.peers_online)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_client_creation() {
        let client = HeartbeatClient::new("https://example.com".to_string());
        assert_eq!(client.worker_url, "https://example.com");
    }
}
