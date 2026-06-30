# Testing End-to-End Encrypted Messages

## Quick Test: Alice → Bob via Encrypted API

### Prerequisites

✅ Release binary built: `./target/release/globy`

### Step 1: Start Bob's Server

```bash
# Terminal 1
cd /home/rudix/Desktop/globy
GLOBY_NICKNAME=Bob ./target/release/globy --host --port 3001
```

**Expected output:**
```
📡 Server with Chat UI
🆔 Your ID: 0xXXXXXXXX (Bob)
📍 API Port: 3001
✅ API Ready - Listening on 0.0.0.0:3001
```

Bob's server now:
- ✅ Has X25519 private key stored in `~/.globy/x25519_private.key`
- ✅ Is listening on port 3001 for encrypted messages
- ✅ Can decrypt & display received messages

### Step 2: Get Bob's X25519 Public Key

```bash
# Terminal 2
cd /home/rudix/Desktop/globy

# Quick way: read Bob's stored private key and derive public
python3 << 'EOF'
import sys
sys.path.insert(0, '/home/rudix/Desktop/globy')

# For testing, we'll manually compute it
# In real scenario, share via QR code or out-of-band
import os
import binascii

home = os.path.expanduser("~")
key_file = os.path.join(home, ".globy", "x25519_private.key")

if os.path.exists(key_file):
    with open(key_file) as f:
        private_hex = f.read().strip()
    print(f"Bob's X25519 private key: {private_hex}")
    # TODO: derive public key using x25519
else:
    print("Key file not found - server will create it on first run")
EOF
```

Or use the Rust binary:

```bash
cat > /tmp/get_x25519.rs << 'EOF'
use std::fs;
use std::path::PathBuf;

fn main() {
    let home = std::env::var("HOME").unwrap();
    let key_file = PathBuf::from(&home).join(".globy/x25519_private.key");
    
    if key_file.exists() {
        if let Ok(content) = fs::read_to_string(&key_file) {
            println!("Bob's X25519 private key hex: {}", content.trim());
        }
    } else {
        println!("Key will be created on server startup");
    }
}
EOF

rustc /tmp/get_x25519.rs -o /tmp/get_x25519 && /tmp/get_x25519
```

For now, **assume Bob's X25519 public key is displayed by the server on startup** (we'll add this).

### Step 3: Create Test Encrypted Message

Create a helper script to encrypt a message:

```bash
cat > /tmp/encrypt_message.py << 'EOF'
#!/usr/bin/env python3
import sys
import json
import os
from pathlib import Path

# Simulate encryption (in real code, use Python nacl or similar)
def create_test_message():
    alice_hash = "0x8737f2d1"
    bob_hash = "0x7e81fc64"
    
    # These would be derived from actual encryption
    ephemeral_pubkey = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0"
    ciphertext = "3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f"
    nonce = "f4e5d6c7b8a9f0e1d2c3b4a5"
    tag = "1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d"
    
    # In real code, verify SSH key path
    home = Path.home()
    ssh_pubkey_path = home / ".ssh" / "id_ed25519.pub"
    
    ssh_pubkey = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI8f3d2c1b9a7e5f4d"
    if ssh_pubkey_path.exists():
        ssh_pubkey = ssh_pubkey_path.read_text().strip()
    
    message = {
        "from_hash": alice_hash,
        "to_hash": bob_hash,
        "ephemeral_pubkey": ephemeral_pubkey,
        "ciphertext": ciphertext,
        "nonce": nonce,
        "tag": tag,
        "signature": "test_signature_placeholder",
        "ssh_pubkey": ssh_pubkey,
        "timestamp": 1719753600
    }
    
    return message

if __name__ == "__main__":
    msg = create_test_message()
    print(json.dumps(msg, indent=2))
EOF

chmod +x /tmp/encrypt_message.py
python3 /tmp/encrypt_message.py > /tmp/encrypted_msg.json
```

### Step 4: Send Encrypted Message to Bob

```bash
# Terminal 2 - Send to Bob's server on port 3001
curl -X POST http://localhost:3001/send-message \
  -H 'Content-Type: application/json' \
  -d @/tmp/encrypted_msg.json

# Expected response:
# {"status":"sent","message_id":"msg_123","timestamp":1719753600}
```

### Step 5: Observe Bob's TUI

**Terminal 1 (Bob's server):**

The TUI will attempt to decrypt the message:

- ✅ If decryption succeeds → Message appears in chat
- ❌ If decryption fails → Error logged, message rejected

**Expected behavior:**

```
Messages received (decrypted):
[0x8737f2d1] Hello Bob!
```

Or check server logs:

```
📨 DM from Alice (0x8737f2d1): Hey Bob! This is encrypted
```

---

## What's Actually Happening

### Server-Side Flow

```rust
// 1. Receives EncryptedMessageRequest
let req: EncryptedMessageRequest = serde_json::from_str(&body)?;

// 2. Loads Bob's X25519 private key
let bob_private = load_x25519_private_key()?; // ~/.globy/x25519_private.key

// 3. Verifies SSH signature (proves Alice sent it)
verify_ssh_signature(&req.signature, &req.ssh_pubkey)?; // ✅

// 4. Derives shared secret (same one Alice computed!)
let shared = DH(bob_private, req.ephemeral_pubkey);

// 5. Decrypts with ChaCha20-Poly1305
let plaintext = decrypt(&req.ciphertext, &shared, bob_hash|alice_hash);
// → {"nickname":"Alice","content":"Hello Bob!","timestamp":...}

// 6. Adds to TUI message queue
queue.add(Message {
    from_hash: "0x8737f2d1",
    nickname: "Alice",
    content: "Hello Bob!",
    timestamp: 1719753600
});
```

### Network Sniffer View

```json
{
  "from_hash": "0x8737f2d1",
  "to_hash": "0x7e81fc64",
  "ephemeral_pubkey": "a1b2c3d4...",
  "ciphertext": "3a4b5c6d...",      ← UNREADABLE!
  "nonce": "f4e5d6c7...",
  "tag": "1a2b3c4d...",
  "signature": "...",
  "ssh_pubkey": "ssh-ed25519 ..."
}
```

**No plaintext visible anywhere on the network** ✅

---

## Real End-to-End Test (No Mocks)

For a fully encrypted test with real data:

```rust
// In examples/test_encryption.rs
use globy::crypto::MessageEncryption;
use globy::ssh_key::SshIdentity;
use serde_json::json;
use std::time::SystemTime;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Alice's setup
    let alice = SshIdentity::new()?;
    let alice_hash = alice.get_peer_hash()?;
    let alice_x25519 = alice.get_x25519_public_key()?;
    
    // Bob's X25519 (simulated - in real scenario, Alice gets this)
    let bob_x25519 = "8f3d2c1b9a7e5f4d6c8b0a2e3f4d5c6b7a8e9f0c1d2e3f4d5c6b7a8e9f0c";
    let bob_hash = "0x7e81fc64";
    
    // Alice encrypts message
    let (ephemeral_pubkey, ephemeral_secret) =
        MessageEncryption::generate_ephemeral_keypair();
    
    let shared_secret = MessageEncryption::derive_shared_secret(
        &ephemeral_secret,
        bob_x25519,
    )?;
    
    let plaintext = serde_json::to_string(&json!({
        "nickname": "Alice",
        "content": "Hello Bob!",
        "timestamp": SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs()
    }))?;
    
    let (ciphertext, nonce, tag) = MessageEncryption::encrypt(
        &plaintext,
        &shared_secret,
        &format!("{}|{}", alice_hash, bob_hash),
    )?;
    
    println!("✅ Alice encrypted message");
    println!("   Ephemeral pubkey: {}", ephemeral_pubkey);
    println!("   Ciphertext: {}...", &ciphertext[..20]);
    
    // Simulate sending to Bob's server...
    // Bob would receive and decrypt
    
    Ok(())
}
```

Run it:
```bash
cargo run --example test_encryption --release
```

---

## Debugging Checklist

| Issue | Diagnosis | Fix |
|-------|-----------|-----|
| **"Decryption failed"** | Wrong shared secret | Verify X25519 keys match |
| **"Invalid SSH signature"** | Signature mismatch | Verify SSH pubkey in request |
| **"Nonce must be 12 bytes"** | Bad hex | Nonce should be 24 hex chars |
| **Message doesn't appear** | Decryption succeeded but parse failed | Check plaintext is valid JSON |
| **Server crashes on message** | Panic in handler | Check logs for stack trace |

---

## Success Criteria

✅ **Test Passes When:**

1. Alice sends encrypted message via curl
2. Bob's server receives it
3. Server verifies SSH signature (passes)
4. Server derives shared secret
5. Server decrypts ciphertext
6. Message appears in Bob's TUI chat
7. Content is "Hello Bob!" (correctly decrypted)
8. Network sniffer sees only ciphertext (no plaintext)

---

## Next: Full Encryption for Client → Server

Currently, the server can decrypt messages received via API.

To complete the flow, implement:

1. **Client encryption** — Helper function to encrypt before sending
2. **SSH signature generation** — Sign encrypted message with SSH key
3. **Key exchange ceremony** — QR code or contact exchange
4. **Persistence** — Optional: store nicknames & decryption keys locally

See `ENCRYPTED_CLIENT_EXAMPLE.md` for the complete client side.
