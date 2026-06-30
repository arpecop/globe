use serde::{Deserialize, Serialize};

/// E2E Encrypted Message Request
/// The message content is encrypted with recipient's public key (X25519)
/// Only the recipient can decrypt it
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessageRequest {
    /// Sender's peer hash (0x8737f2d1)
    pub from_hash: String,

    /// Recipient's peer hash (0x7e81fc64)
    pub to_hash: String,

    /// Ephemeral public key for key exchange (X25519)
    /// Recipient uses this + their private key to derive shared secret
    pub ephemeral_pubkey: String,  // hex-encoded 32 bytes

    /// Encrypted message content + metadata
    /// Format: ChaCha20-Poly1305(plaintext, authenticated_data)
    /// Plaintext: {nickname, content, timestamp}
    pub ciphertext: String,  // hex-encoded

    /// 12-byte nonce for ChaCha20
    pub nonce: String,  // hex-encoded

    /// Authentication tag (proves sender + not tampered)
    pub tag: String,  // hex-encoded (16 bytes)

    /// SSH signature over (to_hash || ephemeral_pubkey || ciphertext)
    /// Proves this came from the claimed sender
    pub signature: String,

    /// Sender's SSH public key (for verification)
    pub ssh_pubkey: String,

    pub timestamp: u64,
}

/// Plain text to encrypt before sending
#[derive(Debug, Serialize, Deserialize)]
pub struct PlaintextMessage {
    pub nickname: String,
    pub content: String,
    pub timestamp: u64,
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
    fn test_encrypted_message_request() {
        let req = EncryptedMessageRequest {
            from_hash: "0x8737f2d1".to_string(),
            to_hash: "0x7e81fc64".to_string(),
            ephemeral_pubkey: "abc123".to_string(),
            ciphertext: "def456".to_string(),
            nonce: "789ghi".to_string(),
            tag: "jkl012".to_string(),
            signature: "sig123".to_string(),
            ssh_pubkey: "ssh-ed25519 ...".to_string(),
            timestamp: 1719753600,
        };
        assert_eq!(req.to_hash, "0x7e81fc64");
    }

    #[test]
    fn test_send_message_response() {
        let resp = SendMessageResponse::success("msg_123".to_string());
        assert_eq!(resp.status, "sent");
    }
}
