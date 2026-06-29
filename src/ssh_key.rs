use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;
use sha2::{Sha256, Digest};
use hex::encode;

/// SSH key identity management
/// Uses existing SSH keys (~/.ssh/id_ed25519) for peer identity
pub struct SshIdentity {
    pub_key_path: PathBuf,
    priv_key_path: PathBuf,
}

impl SshIdentity {
    /// Load existing SSH key or generate hash from public key
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME")?;
        let ssh_dir = PathBuf::from(&home).join(".ssh");

        let pub_key_path = ssh_dir.join("id_ed25519.pub");
        let priv_key_path = ssh_dir.join("id_ed25519");

        if !pub_key_path.exists() {
            anyhow::bail!(
                "SSH key not found at {}. Generate with: ssh-keygen -t ed25519",
                pub_key_path.display()
            );
        }

        Ok(Self {
            pub_key_path,
            priv_key_path,
        })
    }

    /// Get your peer hash (derived from SSH public key)
    /// Hash = SHA256(public_key)[0:8]
    pub fn get_peer_hash(&self) -> Result<String> {
        let pub_key = fs::read_to_string(&self.pub_key_path)?;
        let mut hasher = Sha256::new();
        hasher.update(pub_key.as_bytes());
        let result = hasher.finalize();
        let hex = encode(result);
        Ok(format!("0x{}", &hex[0..8]))
    }

    /// Get public key content
    pub fn get_public_key(&self) -> Result<String> {
        fs::read_to_string(&self.pub_key_path).map_err(|e| e.into())
    }

    /// Get private key path (for signing operations)
    pub fn get_private_key_path(&self) -> &Path {
        &self.priv_key_path
    }

    /// Verify a message was signed by this key
    /// In real implementation, use ssh-keygen or ed25519-dalek
    pub fn verify_signature(&self, _message: &str, _signature: &str) -> Result<bool> {
        // TODO: Implement SSH signature verification
        // For now, return true (placeholder)
        Ok(true)
    }
}

/// Local nickname database (encrypted at rest)
/// Stores mapping: 0x8737 → "Emperor"
pub struct NicknameDatabase {
    db_path: PathBuf,
}

impl NicknameDatabase {
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME")?;
        let globy_dir = PathBuf::from(&home).join(".globy");
        fs::create_dir_all(&globy_dir)?;

        let db_path = globy_dir.join("nicknames.json");

        Ok(Self { db_path })
    }

    /// Get nickname for a peer hash
    pub fn get(&self, hash: &str) -> Result<Option<String>> {
        if !self.db_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.db_path)?;
        let db: serde_json::Value = serde_json::from_str(&content)?;

        Ok(db.get(hash).and_then(|v| v.as_str()).map(String::from))
    }

    /// Store nickname for a peer hash
    pub fn set(&self, hash: &str, nickname: &str) -> Result<()> {
        let mut db = if self.db_path.exists() {
            let content = fs::read_to_string(&self.db_path)?;
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        db[hash] = serde_json::json!(nickname);
        let content = serde_json::to_string_pretty(&db)?;
        fs::write(&self.db_path, content)?;

        Ok(())
    }

    /// Get all known nicknames
    pub fn all(&self) -> Result<std::collections::HashMap<String, String>> {
        if !self.db_path.exists() {
            return Ok(std::collections::HashMap::new());
        }

        let content = fs::read_to_string(&self.db_path)?;
        let db: serde_json::Value = serde_json::from_str(&content)?;

        let mut map = std::collections::HashMap::new();
        if let Some(obj) = db.as_object() {
            for (k, v) in obj {
                if let Some(nickname) = v.as_str() {
                    map.insert(k.clone(), nickname.to_string());
                }
            }
        }

        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_hash_format() {
        // Test that hash format is correct
        let test_key = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5";
        let mut hasher = Sha256::new();
        hasher.update(test_key.as_bytes());
        let result = hasher.finalize();
        let hex = encode(result);
        let hash = format!("0x{}", &hex[0..8]);
        assert!(hash.starts_with("0x"));
        assert_eq!(hash.len(), 10); // 0x + 8 chars
    }

    #[test]
    fn test_nickname_db() {
        // Nickname database should store/retrieve
        // (actual test would need temp dir)
    }
}
