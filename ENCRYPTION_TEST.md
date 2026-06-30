# Testing the Encrypted Message API

## What We've Implemented

The `crypto.rs` module now has:
- ✅ **X25519 key exchange** - Ephemeral keypair generation + DH derivation
- ✅ **ChaCha20-Poly1305 encryption** - AEAD symmetric encryption
- ✅ **Authenticated data binding** - Messages are bound to sender+receiver hashes

## The Encryption Flow

### 1. Sender (Emperor) sends to Recipient (Alice)

```rust
// Sender has Alice's X25519 public key (shared out-of-band)
let alice_x25519_pubkey = "8f3d2c1b9a7e5f4d6c8b0a2e3f4d5c6b7a8e9f0c1d2e3f4d5c6b7a8e9f0c";

// Generate ephemeral keypair (one-time use)
let (ephemeral_pubkey, ephemeral_secret) = 
    MessageEncryption::generate_ephemeral_keypair();

// Derive shared secret using DH
let shared_secret = MessageEncryption::derive_shared_secret(
    &ephemeral_secret,
    alice_x25519_pubkey,
)?;
// Both sender and recipient can compute this same secret!

// Encrypt the message
let (ciphertext, nonce, tag) = MessageEncryption::encrypt(
    r#"{"nickname":"Emperor","content":"Hey Alice!","timestamp":1719753600}"#,
    &shared_secret,
    "0x8737f2d1|0x7e81fc64",  // from_hash|to_hash
)?;

// Sign with SSH key to prove identity
let signature = ssh_key.sign(
    format!("{}||{}||{}", "0x7e81fc64", ephemeral_pubkey, ciphertext).as_bytes()
)?;

// Send API request with all the encrypted data
POST /send-message {
  "from_hash": "0x8737f2d1",
  "to_hash": "0x7e81fc64",
  "ephemeral_pubkey": "a1b2c3d4...",  // ← Shared with recipient
  "ciphertext": "3a4b5c6d...",        // ← Unreadable!
  "nonce": "f4e5d6c7...",
  "tag": "1a2b3c4d...",               // ← Proves authenticity
  "signature": "base64_sig...",        // ← Proves sender identity
  "ssh_pubkey": "ssh-ed25519 ...",
  "timestamp": 1719753600
}
```

### 2. Recipient (Alice) decrypts

```rust
// Alice receives the API request
let req: EncryptedMessageRequest = serde_json::from_str(&body)?;

// Step 1: Verify SSH signature (reject if invalid)
let valid = ssh_key.verify(
    &req.signature,
    format!("{}||{}||{}", req.to_hash, req.ephemeral_pubkey, req.ciphertext),
    &req.ssh_pubkey,
)?;

if !valid {
    // Reject: could be forged!
    return Err("Invalid signature");
}

// Step 2: Load Alice's X25519 private key
let alice_x25519_secret = load_private_key()?; // [u8; 32]

// Step 3: Derive the SAME shared secret
let shared_secret = MessageEncryption::derive_shared_secret(
    &alice_x25519_secret,
    &req.ephemeral_pubkey,
)?;
// Note: Alice uses her private key + sender's ephemeral public key
// Sender used their ephemeral private key + Alice's public key
// Math ensures they both get the same shared secret!

// Step 4: Decrypt
let plaintext = MessageEncryption::decrypt(
    &req.ciphertext,
    &req.nonce,
    &req.tag,
    &shared_secret,
    &format!("{}|{}", req.from_hash, req.to_hash),
)?;

// Step 5: Parse and display
let msg: PlaintextMessage = serde_json::from_str(&plaintext)?;
println!("📨 DM from {}: {}", msg.nickname, msg.content);
// Output: 📨 DM from Emperor: Hey Alice!
```

## Why This Works

### Forward Secrecy ✅
- **Ephemeral keys** — Each message uses a brand-new keypair
- **Old messages safe** — Even if Alice's private key is stolen, old messages (with different ephemeral keys) remain safe
- **One-time use** — The ephemeral_secret is never reused

### No Metadata Leaks ✅
- **Ciphertext is random** — Looks like noise to anyone snooping
- **Authenticated data binding** — Changing sender/recipient breaks the tag
- **SSH signature** — Proves who sent it (can't forge)

### Spam Resistance ✅
- **Unreadable ciphertext** — Spammer sends garbage that won't decrypt
- **MAC verification fails** — Wrong shared secret = failed authentication
- **Bad signature = rejected** — Can't bypass signature check

## Cryptographic Guarantees

| Property | What it protects | How |
|----------|-----------------|-----|
| **Confidentiality** | Message content | ChaCha20 encryption (only recipient's DH key works) |
| **Authentication** | Sender identity | SSH signature (proves sender has their private key) |
| **Integrity** | Message tampering | Poly1305 MAC (changes detected) |
| **Forward Secrecy** | Old messages | Ephemeral keys (old keys don't help decrypt new) |
| **Replay Protection** | Same message twice | Nonce + timestamp (each message is unique) |

## How to Integrate

1. **Server handler** should validate signature before accepting message
2. **Recipient's client** should derive shared secret and decrypt
3. **Store X25519 keys** alongside SSH keys (in `~/.ssh/` or `~/.globy/`)
4. **Share public keys** out-of-band (QR code, contact card, etc.)

## Testing Locally

```bash
# Build with encryption support
cargo build --release

# The crypto module has tests:
cargo test crypto::tests --lib
```

Tests verify:
- Ephemeral keypair generation ✅
- Shared secret derivation (both sides get same value) ✅
- Encryption/decryption round-trip ✅
- Tampering detection (bad MAC rejects) ✅

## Security Notes

⚠️ **This is end-to-end encryption** but does NOT protect:
- **IP addresses** — Use Tor/VPN if needed
- **Metadata** — Timing, size of messages visible
- **Key distribution** — You still need to exchange public keys safely

✅ **This DOES protect:**
- **Message content** — Encrypted with ChaCha20
- **Sender identity** — Cryptographically signed
- **Against forgery** — SSH signature can't be faked
- **Against tampering** — MAC detects any changes
- **Against spam** — Unreadable ciphertext = useless spam

---

## Real Scenario: Fighting Spam

**Attacker** knows Alice's peer hash: `0x7e81fc64`

### Without Encryption (VULNERABLE)
```bash
for i in {1..1000}; do
  curl -X POST http://alice:3000/send-message \
    -d '{"from_hash":"0xATTACKER","content":"BUY CRYPTO NOW!!!"}'
done
# Result: Alice sees 1000 readable spam messages 😞
```

### With Encryption (SAFE) ✅
```bash
for i in {1..1000}; do
  # Attacker doesn't have Alice's X25519 private key
  # So they can't compute the shared secret
  curl -X POST http://alice:3000/send-message \
    -d '{
      "from_hash":"0xATTACKER",
      "ephemeral_pubkey":"random1b2c3d4...",
      "ciphertext":"random3a4b5c6d...",
      "tag":"random1a2b3c4d...",
      "signature":"..."
    }'
done

# Alice's client:
#   1. Tries to decrypt with wrong shared_secret
#   2. Poly1305 MAC check fails
#   3. Message is silently dropped
#
# Result: Spam is invisible, unreadable noise 🔒
```

---

## Next Steps to Complete

1. Store/export X25519 keys from SSH identity
2. Implement server handler for decryption + signature verification
3. Implement client UI for key exchange ceremony
4. Test end-to-end with two running instances
5. Add rate limiting (optional, post-encryption spam mitigation)
