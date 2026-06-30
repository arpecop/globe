# SSH Signature Generation & Verification

## How It Works

Globy now uses SSH keys (ED25519) for cryptographic signing and verification:

### Signing (Sender Side)

```rust
let identity = SshIdentity::new()?;

// Sign the encrypted message
let message_to_sign = b"0x7e81fc64||ephemeral_pubkey||ciphertext";
let signature_hex = identity.sign(message_to_sign)?;

// signature_hex = "abc123def456..." (128 hex chars = 64 bytes)
```

**What happens internally:**

1. Extract ED25519 private key from `~/.ssh/id_ed25519` (OpenSSH format)
2. Call `ssh-keygen -Y sign` to create an armored signature
3. Parse the base64-encoded signature
4. Return as hex string (128 chars for 64-byte ED25519 signature)

### Verification (Receiver Side)

```rust
let identity = SshIdentity::new()?;

let message = "0x7e81fc64||ephemeral_pubkey||ciphertext";
let signature_hex = "abc123def456..."; // from the request
let ssh_pubkey = "ssh-ed25519 AAAAC3..."; // from the request

let is_valid = identity.verify_ssh_signature(
    message,
    signature_hex,
    ssh_pubkey
)?;

if is_valid {
    println!("✅ Signature verified - message is authentic");
} else {
    println!("❌ Bad signature - reject message");
}
```

**What happens internally:**

1. Check that the SSH public key in the request matches ours
2. Decode signature from hex
3. Verify signature was created by the claimed sender

## API Integration

### Before Signing (Client Side)

```rust
use globy::ssh_key::SshIdentity;

async fn send_encrypted_message(to_hash: &str, ciphertext: &str) -> Result<()> {
    let identity = SshIdentity::new()?;
    let from_hash = identity.get_peer_hash()?;
    let ssh_pubkey = identity.get_public_key()?;

    // Build message to sign
    let message_to_sign = format!("{}||{}||{}", to_hash, ephemeral_pubkey, ciphertext);

    // Sign it
    let signature = identity.sign(message_to_sign.as_bytes())?;

    // Send encrypted request with signature
    let request = serde_json::json!({
        "from_hash": from_hash,
        "to_hash": to_hash,
        "ephemeral_pubkey": ephemeral_pubkey,
        "ciphertext": ciphertext,
        "nonce": nonce,
        "tag": tag,
        "signature": signature,  // ← Cryptographic proof
        "ssh_pubkey": ssh_pubkey,
        "timestamp": chrono::Utc::now().timestamp()
    });

    // POST to /send-message
    client.post(url).json(&request).send().await?;
    Ok(())
}
```

### After Signing (Server Side)

```rust
async fn handle_send_message(
    req: EncryptedMessageRequest
) -> Result<Json<SendMessageResponse>> {
    let identity = SshIdentity::new()?;

    // Verify signature BEFORE decrypting
    let message_to_verify = format!(
        "{}||{}||{}",
        req.to_hash,
        req.ephemeral_pubkey,
        req.ciphertext
    );

    let is_valid = identity.verify_ssh_signature(
        &message_to_verify,
        &req.signature,
        &req.ssh_pubkey,
    )?;

    if !is_valid {
        return Err((StatusCode::UNAUTHORIZED, "Invalid signature".to_string()));
    }

    // OK to decrypt and process
    // ...

    Ok(Json(SendMessageResponse::success("msg_123".to_string())))
}
```

## Cryptographic Properties

| Property | Guarantee |
|----------|-----------|
| **Authenticity** | Only holder of private key can create valid signature |
| **Non-repudiation** | Sender can't claim they didn't sign (signature proves it) |
| **Integrity** | If message changes, signature becomes invalid |
| **Unforgeability** | Can't create valid signature without private key |

## Implementation Details

### OpenSSH Key Parsing

Globy parses OpenSSH private key format to extract the ED25519 secret:

```
File: ~/.ssh/id_ed25519
Format:
  -----BEGIN OPENSSH PRIVATE KEY-----
  [base64 encoded key data]
  -----END OPENSSH PRIVATE KEY-----

Inside the base64 data:
  - Magic: "openssh-key-v1\0"
  - Cipher name (usually "none" for unencrypted)
  - KDF name + options
  - Number of keys
  - Public key data
  - Encrypted private key block
    - Checksum
    - Key type ("ssh-ed25519")
    - Public key (32 bytes)
    - Private key (64 bytes: 32-byte seed + 32-byte public)
    - Comment
    - Padding

Globy extracts: the 32-byte ED25519 seed from byte 0-31 of the 64-byte private key
```

### Signature Format

ED25519 signatures are **64 bytes**:
- 32 bytes: r value
- 32 bytes: s value

Globy encodes as **hex** for transport (128 hex characters).

### SSH Signing Tool

For signing, Globy uses `ssh-keygen -Y sign`:

```bash
ssh-keygen -Y sign \
  -f ~/.ssh/id_ed25519 \
  -n globy \
  /tmp/message.txt

# Outputs: /tmp/message.txt.sig
#  Namespace: globy
#  [base64 signature data]
```

## Testing Signature Generation

### Generate a Test Signature

```bash
# Create a test message
echo "hello world" > /tmp/test_msg.txt

# Sign it
ssh-keygen -Y sign \
  -f ~/.ssh/id_ed25519 \
  -n globy \
  /tmp/test_msg.txt

# View the signature
cat /tmp/test_msg.txt.sig
```

Output:
```
-----BEGIN SSH SIGNATURE-----
U1NIU0c=
[base64 data]
-----END SSH SIGNATURE-----
```

### Programmatic Usage

```rust
use globy::ssh_key::SshIdentity;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let identity = SshIdentity::new()?;
    
    // Sign a message
    let message = b"My secret message";
    let signature_hex = identity.sign(message)?;
    
    println!("Message: {:?}", String::from_utf8_lossy(message));
    println!("Signature: {}", signature_hex);
    println!("Sig length: {} (should be 128 hex chars)", signature_hex.len());
    
    // Verify it
    let our_pubkey = identity.get_public_key()?;
    let is_valid = identity.verify_ssh_signature(
        &String::from_utf8_lossy(message),
        &signature_hex,
        &our_pubkey,
    )?;
    
    println!("Valid: {}", is_valid);
    
    Ok(())
}
```

## Security Considerations

### ✅ Strong Points

- Uses **ED25519** (modern, fast, secure elliptic curve)
- **OpenSSH format** (industry standard, widely compatible)
- **SSH-keygen integration** (leverages proven tools)
- **Non-interactive signing** (no passphrase prompts for this use case)
- **Per-message signing** (different signature for each message)

### ⚠️ Limitations

- **Assumes unencrypted SSH keys** (passphrase-protected keys need special handling)
- **Depends on ssh-keygen binary** (must be installed on system)
- **Trust on first use** (need to exchange public keys initially)
- **No key rotation** (compromised key affects all past & future messages)

## Future Improvements

1. **SSH key encryption support** — Handle passphrase-protected keys
2. **Native ed25519 signing** — Don't shell out to ssh-keygen
3. **SSH certificate support** — Use SSH certificates for key rotation
4. **Multi-signature** — Require multiple signers to verify message
5. **Signature timestamp** — Include timestamp in signature for replay protection

---

##Example: Full Signing Flow

```
User A (Alice) sends DM to User B (Bob):

1. Alice composes: "Hello Bob!"
2. Alice's client:
   - Generates ephemeral keypair
   - Derives shared secret with Bob's X25519 pubkey
   - Encrypts message → ciphertext
   - Creates: message_to_sign = "bob_hash||ephemeral||ciphertext"
   - Calls: signature = identity.sign(message_to_sign)
   - Signature: abc123...xyz (128 hex chars, 64 bytes)

3. API Request (POST /send-message):
   {
     "from_hash": "0x8737f2d1",
     "to_hash": "0x7e81fc64",
     "ephemeral_pubkey": "a1b2c3d4...",
     "ciphertext": "3a4b5c6d...",
     "nonce": "f4e5d6c7...",
     "tag": "1a2b3c4d...",
     "signature": "abc123...xyz",         ← Cryptographic proof
     "ssh_pubkey": "ssh-ed25519 AAAA...",
     "timestamp": 1719753600
   }

4. Bob's server receives:
   - Verifies: signature is valid
   - Decrypts: ciphertext → "Hello Bob!"
   - Displays: "Alice: Hello Bob!"
```

---

## Running Tests

```bash
# Build
cargo build --release

# The binary now supports SSH signing for all messages
# Test with:
./target/release/globy --show-key

# Output:
# 🔑 SSH Public Key:
# ssh-ed25519 AAAAC3NzaC1lZDI1NTE5...
# 🆔 Your Peer ID: 0x8737f2d1

# Messages sent via the API will now be cryptographically signed!
```
