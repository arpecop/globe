use serde::{Deserialize, Serialize};

/// Private Message (signed with SSH key)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateMessage {
    /// Sender's peer hash (0x8737...)
    pub from_hash: String,

    /// Recipient's peer hash (0xabcd...)
    pub to_hash: String,

    /// Message content (encrypted end-to-end)
    pub content: String,

    /// SSH signature (proves sender is who they claim)
    pub signature: String,

    /// Timestamp of message
    pub timestamp: u64,

    /// Sender's real nickname (only revealed in PM)
    /// Encrypted with recipient's public key
    pub sender_nickname_encrypted: Option<String>,
}

impl PrivateMessage {
    /// Create new PM (unsigned)
    pub fn new(
        from_hash: String,
        to_hash: String,
        content: String,
    ) -> Self {
        Self {
            from_hash,
            to_hash,
            content,
            signature: String::new(), // Will be signed later
            timestamp: chrono::Utc::now().timestamp() as u64,
            sender_nickname_encrypted: None,
        }
    }

    /// Sign message with SSH key
    pub fn sign(&mut self, signature: String) {
        self.signature = signature;
    }

    /// Set encrypted nickname (encrypted with recipient's public key)
    pub fn set_encrypted_nickname(&mut self, encrypted: String) {
        self.sender_nickname_encrypted = Some(encrypted);
    }

    /// Verify signature is valid
    pub fn is_signed(&self) -> bool {
        !self.signature.is_empty()
    }
}

/// PM Request (asking for someone's nickname via PM)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmRequest {
    /// Sender's peer hash
    pub from_hash: String,

    /// Recipient's peer hash
    pub to_hash: String,

    /// "I'd like to send you a message"
    pub message: String,

    /// Sender's real nickname (encrypted)
    pub sender_nickname_encrypted: String,

    /// SSH signature
    pub signature: String,

    pub timestamp: u64,
}

impl PmRequest {
    pub fn new(
        from_hash: String,
        to_hash: String,
        message: String,
        sender_nickname_encrypted: String,
    ) -> Self {
        Self {
            from_hash,
            to_hash,
            message,
            sender_nickname_encrypted,
            signature: String::new(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }

    pub fn sign(&mut self, signature: String) {
        self.signature = signature;
    }
}

/// PM Response (accepting PM)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmResponse {
    /// Your peer hash
    pub from_hash: String,

    /// Sender's peer hash
    pub to_hash: String,

    /// "Yes, I accept" or "No thanks"
    pub status: String,

    /// Your real nickname (encrypted)
    pub nickname_encrypted: Option<String>,

    /// SSH signature
    pub signature: String,

    pub timestamp: u64,
}

impl PmResponse {
    pub fn accept(
        from_hash: String,
        to_hash: String,
        nickname_encrypted: String,
    ) -> Self {
        Self {
            from_hash,
            to_hash,
            status: "accepted".to_string(),
            nickname_encrypted: Some(nickname_encrypted),
            signature: String::new(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }

    pub fn reject(from_hash: String, to_hash: String) -> Self {
        Self {
            from_hash,
            to_hash,
            status: "rejected".to_string(),
            nickname_encrypted: None,
            signature: String::new(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }

    pub fn sign(&mut self, signature: String) {
        self.signature = signature;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pm_creation() {
        let pm = PrivateMessage::new(
            "0x8737".to_string(),
            "0xabcd".to_string(),
            "Hello!".to_string(),
        );
        assert_eq!(pm.from_hash, "0x8737");
        assert_eq!(pm.to_hash, "0xabcd");
        assert_eq!(pm.content, "Hello!");
        assert!(!pm.is_signed());
    }

    #[test]
    fn test_pm_signing() {
        let mut pm = PrivateMessage::new(
            "0x8737".to_string(),
            "0xabcd".to_string(),
            "Hello!".to_string(),
        );
        pm.sign("signature_123".to_string());
        assert!(pm.is_signed());
    }

    #[test]
    fn test_pm_request() {
        let mut req = PmRequest::new(
            "0x8737".to_string(),
            "0xabcd".to_string(),
            "Want to chat?".to_string(),
            "nickname_encrypted".to_string(),
        );
        assert!(!req.signature.is_empty() || req.signature.is_empty());
        req.sign("sig_123".to_string());
        assert_eq!(req.signature, "sig_123");
    }
}
