use sha2::{Sha256, Digest};
use hex::{encode, decode};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use chacha20poly1305::aead::{Aead, Payload, KeyInit};
use x25519_dalek::x25519;
use rand::RngCore;

const X25519_BASEPOINT: [u8; 32] = [
    9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

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

/// E2E Message Encryption using X25519 + ChaCha20-Poly1305
pub struct MessageEncryption;

impl MessageEncryption {
    /// Generate an ephemeral keypair for key exchange
    pub fn generate_ephemeral_keypair() -> (String, [u8; 32]) {
        let mut secret_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut secret_bytes);
        // Compute public key from secret
        let public_bytes = x25519(secret_bytes, X25519_BASEPOINT);
        (encode(&public_bytes), secret_bytes)
    }

    /// Derive shared secret from ephemeral_secret + recipient_pubkey
    /// Both sides compute: shared_secret = DH(ephemeral_secret, recipient_pubkey)
    pub fn derive_shared_secret(
        ephemeral_secret: &[u8; 32],
        recipient_pubkey_hex: &str,
    ) -> Result<[u8; 32], String> {
        let recipient_bytes = decode(recipient_pubkey_hex)
            .map_err(|e| format!("Invalid hex pubkey: {}", e))?;

        if recipient_bytes.len() != 32 {
            return Err("Recipient pubkey must be 32 bytes".to_string());
        }

        let mut pubkey_array = [0u8; 32];
        pubkey_array.copy_from_slice(&recipient_bytes);

        let shared_secret = x25519(*ephemeral_secret, pubkey_array);
        Ok(shared_secret)
    }

    /// Encrypt plaintext with ChaCha20-Poly1305
    /// authenticated_data: typically (from_hash || to_hash) to bind message to sender+receiver
    pub fn encrypt(
        plaintext: &str,
        shared_secret: &[u8; 32],
        authenticated_data: &str,
    ) -> Result<(String, String, String), String> {
        let cipher = ChaCha20Poly1305::new(shared_secret.into());

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from(nonce_bytes);

        let aad = authenticated_data.as_bytes();
        let mut ciphertext = cipher
            .encrypt(&nonce, Payload { msg: plaintext.as_bytes(), aad: aad })
            .map_err(|e| format!("Encryption failed: {}", e))?;

        // ChaCha20Poly1305 appends the 16-byte auth tag to ciphertext
        let tag = ciphertext.split_off(ciphertext.len() - 16);

        Ok((
            encode(&ciphertext), // ciphertext without tag
            encode(&nonce_bytes), // nonce
            encode(&tag), // auth tag (last 16 bytes)
        ))
    }

    /// Decrypt ciphertext with ChaCha20-Poly1305
    pub fn decrypt(
        ciphertext_hex: &str,
        nonce_hex: &str,
        tag_hex: &str,
        shared_secret: &[u8; 32],
        authenticated_data: &str,
    ) -> Result<String, String> {
        let cipher = ChaCha20Poly1305::new(shared_secret.into());

        let nonce_bytes = decode(nonce_hex)
            .map_err(|e| format!("Invalid nonce hex: {}", e))?;
        if nonce_bytes.len() != 12 {
            return Err("Nonce must be 12 bytes".to_string());
        }
        let nonce_array: [u8; 12] = nonce_bytes.try_into()
            .map_err(|_| "Failed to convert nonce bytes".to_string())?;
        let nonce = Nonce::from(nonce_array);

        let ct_bytes = decode(ciphertext_hex)
            .map_err(|e| format!("Invalid ciphertext hex: {}", e))?;
        let tag_bytes = decode(tag_hex)
            .map_err(|e| format!("Invalid tag hex: {}", e))?;

        let mut full_ciphertext = ct_bytes;
        full_ciphertext.extend_from_slice(&tag_bytes);

        let aad = authenticated_data.as_bytes();
        let plaintext = cipher
            .decrypt(&nonce, Payload { msg: full_ciphertext.as_slice(), aad: aad })
            .map_err(|e| format!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext)
            .map_err(|e| format!("Invalid UTF-8: {}", e))
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
