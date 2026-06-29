use sha2::{Sha256, Digest};
use hex::encode;

pub struct NicknameHasher {
    salt: String,
}

impl NicknameHasher {
    pub fn new(salt: String) -> Self {
        Self { salt }
    }

    /// Hash nickname deterministically: SHA256(nickname|device_id|salt)
    /// Always returns same hash for same inputs
    /// Result: 0x + 8 hex chars (e.g., 0x8737f2d1)
    pub fn hash(&self, nickname: &str, device_id: &str) -> String {
        let input = format!("{}|{}|{}", nickname, device_id, self.salt);
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        let hex = encode(result);
        format!("0x{}", &hex[0..8])
    }

    /// Verify that a given hash matches the nickname + device_id
    pub fn verify(&self, nickname: &str, device_id: &str, hash: &str) -> bool {
        self.hash(nickname, device_id) == hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_hashing() {
        let hasher = NicknameHasher::new("test_salt".to_string());
        let hash1 = hasher.hash("Emperor", "device_123");
        let hash2 = hasher.hash("Emperor", "device_123");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_device_different_hash() {
        let hasher = NicknameHasher::new("test_salt".to_string());
        let hash1 = hasher.hash("Emperor", "device_123");
        let hash2 = hasher.hash("Emperor", "device_456");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_different_nickname_different_hash() {
        let hasher = NicknameHasher::new("test_salt".to_string());
        let hash1 = hasher.hash("Emperor", "device_123");
        let hash2 = hasher.hash("King", "device_123");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_format() {
        let hasher = NicknameHasher::new("test_salt".to_string());
        let hash = hasher.hash("Emperor", "device_123");
        assert!(hash.starts_with("0x"));
        assert_eq!(hash.len(), 10); // 0x + 8 chars
    }
}
