use serde::{Deserialize, Serialize};

/// Handshake Message (no encryption needed for P2P)
/// TLS handles transport security, SSH key handles auth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeMessage {
    /// Sender's peer hash (0x8737...)
    pub from_hash: String,

    /// Sender's nickname (public in handshake)
    pub nickname: String,

    /// Sender's IP (for P2P connection)
    pub ip: String,

    /// Sender's port
    pub port: u16,

    /// SSH public key (for verification)
    pub ssh_pubkey: String,

    /// SSH signature (proves authenticity)
    pub signature: String,

    pub timestamp: u64,
}

impl HandshakeMessage {
    pub fn new(
        from_hash: String,
        nickname: String,
        ip: String,
        port: u16,
        ssh_pubkey: String,
    ) -> Self {
        Self {
            from_hash,
            nickname,
            ip,
            port,
            ssh_pubkey,
            signature: String::new(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }

    pub fn sign(&mut self, signature: String) {
        self.signature = signature;
    }

    pub fn is_signed(&self) -> bool {
        !self.signature.is_empty()
    }
}

/// Message (plaintext, sent P2P over TLS)
/// No message-level encryption needed if P2P works
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub from_hash: String,
    pub from_nickname: String,
    pub content: String,
    pub timestamp: u64,
    pub signature: String,
}

impl ChatMessage {
    pub fn new(from_hash: String, from_nickname: String, content: String) -> Self {
        Self {
            from_hash,
            from_nickname,
            content,
            timestamp: chrono::Utc::now().timestamp() as u64,
            signature: String::new(),
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
    fn test_handshake_creation() {
        let hs = HandshakeMessage::new(
            "0x8737".to_string(),
            "Emperor".to_string(),
            "1.2.3.4".to_string(),
            3000,
            "ssh-ed25519 ...".to_string(),
        );
        assert_eq!(hs.from_hash, "0x8737");
        assert_eq!(hs.nickname, "Emperor");
        assert!(!hs.is_signed());
    }

    #[test]
    fn test_chat_message() {
        let msg = ChatMessage::new(
            "0x8737".to_string(),
            "Emperor".to_string(),
            "Hello!".to_string(),
        );
        assert_eq!(msg.from_hash, "0x8737");
        assert_eq!(msg.content, "Hello!");
    }
}
