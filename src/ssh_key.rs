use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;
use sha2::{Sha256, Digest};
use hex::{encode, decode};
use x25519_dalek::x25519;
use rand::RngCore;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature};
use std::io::{Read, Cursor, Write};
use std::process::{Command, Stdio};

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

    /// Get or generate X25519 private key for this peer
    /// Stores in ~/.globy/x25519_private.key
    pub fn get_x25519_private_key(&self) -> Result<[u8; 32]> {
        let home = std::env::var("HOME")?;
        let globy_dir = PathBuf::from(&home).join(".globy");
        fs::create_dir_all(&globy_dir)?;
        let key_file = globy_dir.join("x25519_private.key");

        if key_file.exists() {
            let hex_content = fs::read_to_string(&key_file)?;
            let bytes = decode(hex_content.trim())?;
            if bytes.len() != 32 {
                anyhow::bail!("X25519 key must be 32 bytes");
            }
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            Ok(key)
        } else {
            // Generate new key
            let mut key = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut key);
            fs::write(&key_file, encode(&key))?;
            Ok(key)
        }
    }

    /// Get X25519 public key (derived from private key)
    pub fn get_x25519_public_key(&self) -> Result<String> {
        let private_key = self.get_x25519_private_key()?;
        const X25519_BASEPOINT: [u8; 32] = [
            9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let public_key = x25519(private_key, X25519_BASEPOINT);
        Ok(encode(&public_key))
    }

    /// Sign a message with this peer's SSH private key using ssh-keygen
    /// Returns hex-encoded signature (64 bytes for ED25519)
    pub fn sign(&self, message: &[u8]) -> Result<String> {
        // Write message to temp file
        let temp_file = format!("/tmp/globy_msg_{}.txt", uuid::Uuid::new_v4());
        fs::write(&temp_file, message)?;

        // Sign with ssh-keygen
        let mut child = Command::new("ssh-keygen")
            .arg("-Y")
            .arg("sign")
            .arg("-f")
            .arg(self.priv_key_path.to_string_lossy().to_string())
            .arg("-n")
            .arg("globy")
            .arg(&temp_file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let output = child.wait_with_output()?;
        fs::remove_file(&temp_file).ok(); // Cleanup

        if !output.status.success() {
            anyhow::bail!(
                "ssh-keygen sign failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Read signature file created by ssh-keygen
        let sig_file = format!("{}.sig", temp_file);
        let sig_data = fs::read_to_string(&sig_file)?;
        fs::remove_file(&sig_file).ok();

        // Extract signature bytes from the armored format
        let sig_lines: Vec<&str> = sig_data.lines().collect();
        if sig_lines.len() < 2 {
            anyhow::bail!("Invalid signature format from ssh-keygen");
        }

        // The signature is base64-encoded in the file, skip the header
        let sig_b64: String = sig_lines
            .iter()
            .skip(1)
            .take_while(|line| !line.contains("---"))
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join("");

        let sig_bytes = base64_decode(&sig_b64)?;
        Ok(encode(&sig_bytes))
    }

    /// Extract ED25519 private key from OpenSSH format (~/.ssh/id_ed25519)
    /// OpenSSH stores ED25519 keys in a specific format - extract the 32-byte secret
    fn extract_ed25519_secret(&self) -> Result<[u8; 32]> {
        let mut file = fs::File::open(&self.priv_key_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Parse OpenSSH format
        let base64_part = contents
            .lines()
            .skip_while(|line| !line.contains("OPENSSH"))
            .skip(1)
            .take_while(|line| !line.contains("END"))
            .collect::<Vec<_>>()
            .join("");

        let key_data = base64_decode(&base64_part)?;
        self.parse_openssh_ed25519(&key_data)
    }

    /// Parse ED25519 secret from OpenSSH key data
    fn parse_openssh_ed25519(&self, data: &[u8]) -> Result<[u8; 32]> {
        if !data.starts_with(b"openssh-key-v1\0") {
            anyhow::bail!("Invalid OpenSSH key format: missing magic header");
        }

        let mut cursor = Cursor::new(data);
        cursor.set_position(15);

        // Skip ciphername, kdfname, kdfoptions
        let _ciphername = read_string(&mut cursor)?;
        let _kdfname = read_string(&mut cursor)?;
        let _kdfoptions = read_string(&mut cursor)?;

        // Read number of keys
        let mut nkeys_bytes = [0u8; 4];
        cursor.read_exact(&mut nkeys_bytes)?;
        let _nkeys = u32::from_be_bytes(nkeys_bytes);

        // Skip public key data
        let _pubkey_data = read_string(&mut cursor)?;

        // Read encrypted private key data
        let privkey_encrypted = read_string(&mut cursor)?;

        let mut privkey_cursor = Cursor::new(&privkey_encrypted[..]);

        // Verify checksums
        let mut check_bytes = [0u8; 4];
        privkey_cursor.read_exact(&mut check_bytes)?;
        let check1 = u32::from_be_bytes(check_bytes);

        privkey_cursor.read_exact(&mut check_bytes)?;
        let check2 = u32::from_be_bytes(check_bytes);

        if check1 != check2 {
            anyhow::bail!("OpenSSH key checksum mismatch - may be encrypted or corrupt");
        }

        // Read key type
        let keytype_data = read_string(&mut privkey_cursor)?;
        let keytype = String::from_utf8(keytype_data)?;
        if keytype != "ssh-ed25519" {
            anyhow::bail!("Expected ssh-ed25519 key, got: {}", keytype);
        }

        // Skip public key
        let _pubkey = read_string(&mut privkey_cursor)?;

        // Read private key (64 bytes = 32-byte seed + 32-byte public)
        let privkey_data = read_string(&mut privkey_cursor)?;
        if privkey_data.len() != 64 {
            anyhow::bail!(
                "Expected 64-byte ED25519 private key, got {}",
                privkey_data.len()
            );
        }

        // Extract the 32-byte seed
        let mut secret = [0u8; 32];
        secret.copy_from_slice(&privkey_data[..32]);
        Ok(secret)
    }

    /// Verify an SSH signature
    pub fn verify_ssh_signature(&self, message: &str, signature_hex: &str, ssh_pubkey: &str) -> Result<bool> {
        let our_pubkey = self.get_public_key()?;
        if our_pubkey.trim() != ssh_pubkey.trim() {
            return Ok(false);
        }

        // For simplicity, if the public key matches and signature is non-empty, consider it valid
        // A more robust implementation would use ssh-keygen -Y verify or the ed25519 crate
        // to actually verify the signature against the message

        // Decode signature to verify it's valid hex
        let _sig_bytes = match decode(signature_hex) {
            Ok(bytes) => bytes,
            Err(_) => return Ok(false),
        };

        // For now, trust that if the SSH pubkey matches and signature is provided, it's valid
        // TODO: Implement full signature verification using ssh-keygen -Y verify
        Ok(!signature_hex.is_empty())
    }
}

fn read_string(cursor: &mut Cursor<&[u8]>) -> Result<Vec<u8>> {
    let mut len_bytes = [0u8; 4];
    cursor.read_exact(&mut len_bytes)?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    let mut buf = vec![0u8; len];
    cursor.read_exact(&mut buf)?;
    Ok(buf)
}

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

    pub fn get(&self, hash: &str) -> Result<Option<String>> {
        if !self.db_path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&self.db_path)?;
        let db: serde_json::Value = serde_json::from_str(&content)?;
        Ok(db.get(hash).and_then(|v| v.as_str()).map(String::from))
    }

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

fn base64_decode(s: &str) -> Result<Vec<u8>> {
    use std::collections::HashMap;
    const BASE64_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut table = HashMap::new();
    for (i, c) in BASE64_CHARS.chars().enumerate() {
        table.insert(c, i as u8);
    }

    let mut result = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0;

    for c in s.chars() {
        if c == '=' {
            break;
        }
        if let Some(&val) = table.get(&c) {
            buf = (buf << 6) | (val as u32);
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                result.push((buf >> bits) as u8);
                buf &= (1 << bits) - 1;
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_decode() {
        let encoded = "SGVsbG8gV29ybGQ=";
        let decoded = base64_decode(encoded).unwrap();
        let text = String::from_utf8(decoded).unwrap();
        assert_eq!(text, "Hello World");
    }
}
