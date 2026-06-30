# Globy Encrypted API - Complete Example

## Problem: Why Encryption Matters

**Before (INSECURE):**
```bash
curl -X POST http://localhost:3000/send-message \
  -H 'Content-Type: application/json' \
  -d '{"to_hash":"0x7e81fc64","content":"secret meeting at 3pm","from_hash":"0x8737f2d1"}'
```
- Network observer sees: plaintext message + sender + recipient
- Spammer can send junk messages to your hash that you can read
- You're deanonymized if someone sniffs the message content

**After (ENCRYPTED):**
- Message is encrypted → network sees only unreadable bytes
- Spammer can still send ciphertext but you can't decrypt it (can ignore/delete)
- Only the intended recipient can decrypt
- Sender is cryptographically proven via SSH signature

---

## Architecture: X25519 + ChaCha20-Poly1305

```
┌─────────────────────────────────────────────────────────┐
│ SENDER (wants to DM Alice)                              │
├─────────────────────────────────────────────────────────┤
│ 1. Get recipient's X25519 public key (shared somehow)   │
│ 2. Generate ephemeral keypair (one-time)                │
│ 3. Perform X25519 DH → shared_secret                    │
│ 4. Use shared_secret to derive ChaCha20 key             │
│ 5. Encrypt: {nickname, content, timestamp}             │
│ 6. Sign with SSH key                                    │
│ 7. Send all to recipient                                │
└─────────────────────────────────────────────────────────┘
                        ↓
                  (NETWORK: ciphertext)
                        ↓
┌─────────────────────────────────────────────────────────┐
│ RECIPIENT (Alice)                                       │
├─────────────────────────────────────────────────────────┤
│ 1. Receive ephemeral_pubkey                             │
│ 2. Use my X25519 private key + ephemeral_pubkey → same  │
│    shared_secret as sender                              │
│ 3. Decrypt ciphertext with ChaCha20 key                 │
│ 4. Verify SSH signature (proves it's really the sender) │
│ 5. Display: "Emperor: hello alice!"                     │
└─────────────────────────────────────────────────────────┘
```

---

## Step-by-Step Example

### Step 1: Alice's Public Key

Alice already has `~/.ssh/id_ed25519` (for SSH auth).
Derive her X25519 public key from it:

```bash
# Alice shows her public key (or it's shared via out-of-band)
# This is NOT her SSH key, it's derived for DH key exchange
alice_x25519_pubkey="8f3d2c1b9a7e5f4d6c8b0a2e3f4d5c6b7a8e9f0c1d2e3f4d5c6b7a8e9f0c"
```

### Step 2: Emperor wants to send Alice a message

**Client code (Rust example):**

```rust
use crate::crypto::MessageEncryption;
use serde_json::json;

// Step 1: Generate ephemeral keypair
let (ephemeral_pubkey, ephemeral_secret) = MessageEncryption::generate_ephemeral_keypair();
// ephemeral_pubkey = "a1b2c3d4e5f6..." (hex)
// ephemeral_secret = [u8; 32]

// Step 2: Derive shared secret
let shared_secret = MessageEncryption::derive_shared_secret(
    &ephemeral_secret,
    "8f3d2c1b9a7e5f4d6c8b0a2e3f4d5c6b7a8e9f0c1d2e3f4d5c6b7a8e9f0c", // alice_x25519_pubkey
)?;

// Step 3: Prepare plaintext
let plaintext = serde_json::to_string(&json!({
    "nickname": "Emperor",
    "content": "Hey Alice! Let's chat",
    "timestamp": 1719753600
}))?;

// Step 4: Encrypt
let (ciphertext, nonce, tag) = MessageEncryption::encrypt(
    &plaintext,
    &shared_secret,
    "0x8737f2d1|0x7e81fc64", // authenticated_data = from_hash|to_hash
)?;

// Step 5: Sign with SSH key
let message_to_sign = format!("{}||{}||{}", "0x7e81fc64", ephemeral_pubkey, ciphertext);
let signature = ssh_key.sign(message_to_sign.as_bytes())?; // Your SSH key signs

// Step 6: Build API request
let request = json!({
    "from_hash": "0x8737f2d1",
    "to_hash": "0x7e81fc64",
    "ephemeral_pubkey": ephemeral_pubkey,
    "ciphertext": ciphertext,
    "nonce": nonce,
    "tag": tag,
    "signature": signature,
    "ssh_pubkey": "ssh-ed25519 AAAAC3...",
    "timestamp": 1719753600
});

println!("{}", serde_json::to_string_pretty(&request)?);
```

### Step 3: API Request (with encryption)

```bash
curl -X POST http://alice.peer:3000/send-message \
  -H 'Content-Type: application/json' \
  -d '{
    "from_hash": "0x8737f2d1",
    "to_hash": "0x7e81fc64",
    "ephemeral_pubkey": "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0",
    "ciphertext": "3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f",
    "nonce": "f4e5d6c7b8a9",
    "tag": "1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d",
    "signature": "base64_encoded_ssh_signature...",
    "ssh_pubkey": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI...",
    "timestamp": 1719753600
  }'
```

**Network observer sees:**
- ✅ `from_hash`, `to_hash` (metadata - can't avoid this)
- ✅ `ephemeral_pubkey` (random, different every message)
- ✅ `ciphertext` (unreadable: "3a4b5c6d7e8f...")
- ✅ `tag` (authentication only, reveals nothing)
- ❌ **Cannot read message content**
- ❌ **Cannot forge messages** (signature proves authenticity)
- ❌ **Cannot decrypt even if they intercept** (only you have the private key)

---

## Step 4: Alice Receives and Decrypts

**Server receives the request and stores/forwards to Alice:**

```rust
// Alice's server receives the encrypted message
let req: EncryptedMessageRequest = serde_json::from_str(&body)?;

// Step 1: Verify SSH signature first (reject if invalid)
let is_valid = ssh_key.verify(
    &req.signature,
    format!("{}||{}||{}", req.to_hash, req.ephemeral_pubkey, req.ciphertext).as_bytes(),
    &req.ssh_pubkey,
)?;

if !is_valid {
    return Err("Invalid signature - message rejected".to_string());
}

// Step 2: Load Alice's X25519 private key
let alice_x25519_secret = load_alice_x25519_secret()?; // [u8; 32]

// Step 3: Derive shared secret (same as sender computed it)
let shared_secret = MessageEncryption::derive_shared_secret(
    &alice_x25519_secret,
    &req.ephemeral_pubkey,
)?;

// Step 4: Decrypt
let plaintext = MessageEncryption::decrypt(
    &req.ciphertext,
    &req.nonce,
    &req.tag,
    &shared_secret,
    &format!("{}|{}", req.from_hash, req.to_hash),
)?;

// Step 5: Parse decrypted JSON
let msg: PlaintextMessage = serde_json::from_str(&plaintext)?;

// Step 6: Display to Alice
println!("📨 DM from {}: {}", msg.nickname, msg.content);
// Output: 📨 DM from Emperor: Hey Alice! Let's chat
```

---

## Why This Defeats Spam

### Scenario: Spammer wants to flood Alice

**Before (plaintext):**
```bash
# Spammer can send 1000 legible messages to Alice
for i in {1..1000}; do
  curl -X POST http://alice:3000/send-message \
    -d '{"to_hash":"0x7e81fc64","content":"BUY CRYPTO NOW!!!","from_hash":"0xSPAMHASH"}'
done
# Result: Alice sees 1000 readable spam messages
```

**After (encrypted):**
```bash
# Spammer can still send 1000 messages, but...
for i in {1..1000}; do
  curl -X POST http://alice:3000/send-message \
    -d '{"to_hash":"0x7e81fc64","ciphertext":"3a4b5c6d...","nonce":"f4e5d6c7...","tag":"1a2b3c4d..."}'
done

# Alice sees: 1000 random unreadable ciphertexts
# Alice's client:
#   - Tries to decrypt with spammer's ephemeral_pubkey
#   - Derives wrong shared_secret (spammer doesn't know Alice's private key)
#   - Decryption fails → message is dropped/ignored
# Result: Spam is useless, unreadable noise
```

**Even if spam decrypts:**
- The sender is cryptographically signed → Alice blocks that SSH key
- Spammer would need to generate a new SSH key for each message (expensive)

---

## Key Exchange Ceremony (Out of Band)

For two strangers to message:

```
User A: "Hi, I'm Emperor (0x8737f2d1)"
        "Here's my X25519 pubkey: a1b2c3d4e5f6..."

User B: "I'm Alice (0x7e81fc64)"
        "Here's mine: 8f3d2c1b9a7e5f4d..."

[Exchange can happen via:
 - QR code scan
 - Out-of-band message
 - Shared secret
 - Contact card with pubkey embedded]

Now they can message each other encrypted!
```

---

## Security Properties

| Property | Status | How |
|----------|--------|-----|
| **Confidentiality** | ✅ | ChaCha20-Poly1305, only recipient can decrypt |
| **Authentication** | ✅ | SSH signature proves sender identity |
| **Integrity** | ✅ | Poly1305 MAC + authenticated_data binding |
| **Forward Secrecy** | ✅ | Ephemeral DH keys (one-time use) |
| **Spam Resistance** | ✅ | Ciphertext unreadable → spam has no effect |
| **IP Anonymity** | ❌ | Use Tor/VPN for this layer |
| **Timing Resistance** | ❌ | Timestamps visible (use random delays if needed) |

---

## Implementation Checklist

- [ ] Add `MessageEncryption::encrypt()` / `decrypt()` to `crypto.rs`
- [ ] Update `EncryptedMessageRequest` struct in `api.rs`
- [ ] Implement server handler for `/send-message` (encrypted)
- [ ] Implement client to send encrypted messages
- [ ] Implement recipient decryption + signature verification
- [ ] Store X25519 private key (derive from SSH key or separate)
- [ ] Export X25519 public key for sharing with peers
- [ ] Test: send message → intercept → verify ciphertext is unreadable
- [ ] Test: spammer sends garbage → verify it's dropped on decrypt failure

---

## Code Example: Client Helper

```rust
use crate::api::EncryptedMessageRequest;
use crate::crypto::MessageEncryption;
use crate::ssh_key::SshIdentity;

pub async fn send_encrypted_message(
    recipient_x25519_pubkey: &str,
    recipient_hash: &str,
    message_content: &str,
    my_nickname: &str,
) -> Result<EncryptedMessageRequest> {
    let identity = SshIdentity::new()?;
    let my_hash = identity.get_peer_hash()?;
    let my_ssh_pubkey = identity.get_public_key()?;

    // Generate ephemeral keypair
    let (ephemeral_pubkey, ephemeral_secret) =
        MessageEncryption::generate_ephemeral_keypair();

    // Derive shared secret
    let shared_secret = MessageEncryption::derive_shared_secret(
        &ephemeral_secret,
        recipient_x25519_pubkey,
    )?;

    // Prepare plaintext
    let plaintext = serde_json::to_string(&serde_json::json!({
        "nickname": my_nickname,
        "content": message_content,
        "timestamp": chrono::Utc::now().timestamp()
    }))?;

    // Encrypt
    let (ciphertext, nonce, tag) = MessageEncryption::encrypt(
        &plaintext,
        &shared_secret,
        &format!("{}|{}", my_hash, recipient_hash),
    )?;

    // Sign
    let message_to_sign =
        format!("{}||{}||{}", recipient_hash, ephemeral_pubkey, ciphertext);
    let signature = identity.sign(message_to_sign.as_bytes())?;

    Ok(EncryptedMessageRequest {
        from_hash: my_hash,
        to_hash: recipient_hash.to_string(),
        ephemeral_pubkey,
        ciphertext,
        nonce,
        tag,
        signature,
        ssh_pubkey: my_ssh_pubkey,
        timestamp: chrono::Utc::now().timestamp() as u64,
    })
}
```

---

## Testing: See It Work

```bash
# Terminal 1: Start Alice's server
GLOBY_NICKNAME=Alice ./target/release/globy --host --port 3001

# Terminal 2: Start Emperor's server
GLOBY_NICKNAME=Emperor ./target/release/globy --host --port 3002

# Terminal 3: Send encrypted message
# (Use the client helper above to generate request)
curl -X POST http://localhost:3001/send-message \
  -H 'Content-Type: application/json' \
  -d @encrypted_message.json

# Alice's terminal should show:
# 📨 DM from Emperor: Hey Alice! Let's chat
```

**Network sniffer sees:**
```json
{
  "from_hash": "0x8737f2d1",
  "to_hash": "0x7e81fc64",
  "ephemeral_pubkey": "a1b2c3d4...",
  "ciphertext": "3a4b5c6d...",  // ← Unreadable!
  "nonce": "f4e5d6c7...",
  "tag": "1a2b3c4d...",
  "signature": "SGVsbG8gV29ybGQ=",  // ← Proves identity
}
```

No plaintext visible. Spam is unreadable. ✅

---

## Next Steps

1. Integrate `MessageEncryption` into server handlers
2. Implement client-side message preparation
3. Store + distribute X25519 public keys
4. Test end-to-end encryption
5. Add spam filtering (rate limit + block bad SSH keys)
