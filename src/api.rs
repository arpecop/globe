use serde::{Deserialize, Serialize};

/// API: Send message to peer
#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    /// Recipient peer hash (0x7e81fc64)
    pub to_hash: String,

    /// Message content
    pub content: String,

    /// Sender's peer hash (optional, can be inferred)
    pub from_hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub status: String,
    pub message_id: String,
    pub timestamp: u64,
}

impl SendMessageResponse {
    pub fn success(message_id: String) -> Self {
        Self {
            status: "sent".to_string(),
            message_id,
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }

    pub fn failed(reason: &str) -> Self {
        Self {
            status: format!("failed: {}", reason),
            message_id: String::new(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }
}

/// API: Get messages for peer
#[derive(Debug, Serialize, Deserialize)]
pub struct GetMessagesRequest {
    pub from_hash: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetMessagesResponse {
    pub messages: Vec<MessageData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageData {
    pub id: String,
    pub from_hash: String,
    pub content: String,
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_message_request() {
        let req = SendMessageRequest {
            to_hash: "0x7e81fc64".to_string(),
            content: "hello".to_string(),
            from_hash: Some("0x8737f2d1".to_string()),
        };
        assert_eq!(req.to_hash, "0x7e81fc64");
    }

    #[test]
    fn test_send_message_response() {
        let resp = SendMessageResponse::success("msg_123".to_string());
        assert_eq!(resp.status, "sent");
    }
}
