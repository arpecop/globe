# Globy SSH Terminal Service

## The Problem We're Solving

Currently, Globy requires:
- ❌ Direct connections between peers (needs public IP)
- ❌ Port forwarding setup
- ❌ NAT traversal
- ❌ Firewall rules

**This doesn't work for:**
- Uganda Bob behind home router
- Users behind corporate firewalls  
- Mobile users on cellular networks
- Anyone without a public IP

## The Solution: SSH Terminal Service

Make Globy like **Terminal.ssh** or **Eternal Terminal** - users SSH into a relay server to access their secure, encrypted chat.

```
┌─────────────────────────────────────────────┐
│   Globy SSH Terminal Service                │
│   @ 130.204.65.82:2222                      │
├─────────────────────────────────────────────┤
│                                             │
│  $ ssh alice@130.204.65.82 -p 2222         │
│  > [Globy TUI Appears]                      │
│                                             │
│  $ ssh bob@130.204.65.82 -p 2222           │
│  > [Globy TUI Appears]                      │
│                                             │
│  Both running encrypted E2E chat            │
│  Relay can't read messages                  │
│                                             │
└─────────────────────────────────────────────┘
```

## Architecture

### Component Stack

```
User Terminal
     ↓ (SSH connection)
SSH Transport (russh)
     ↓
Session Handler (authenticates SSH key)
     ↓
TUI Chat Interface (ratatui)
     ↓ (encrypted messages)
Message Queue (relay)
     ↓ (routes encrypted blobs)
Other SSH Sessions
```

### Authentication

Uses existing SSH keys at `~/.ssh/id_ed25519`:

```
User runs: ssh alice@130.204.65.82
           ↓
SSH server checks: authorized_keys or public key auth
           ↓
TUI launches with username='alice'
           ↓
User can now:
  - Type messages
  - Receive encrypted messages from other peers
  - All E2E encrypted (relay can't read)
```

## Key Benefits

✅ **No public IP needed** - Connect to relay from anywhere  
✅ **NAT transparent** - Works behind any firewall  
✅ **Mobile friendly** - Works on cellular, VPN, etc.  
✅ **Existing auth** - Uses SSH keys users already have  
✅ **E2E encrypted** - Relay can't read messages  
✅ **Persistent sessions** - Reconnect anytime  
✅ **Simple deployment** - One SSH server on relay  
✅ **Works everywhere SSH works**  

## Implementation Plan

### Phase 1: SSH Server (Now)
- [ ] Accept SSH connections on port 2222
- [ ] Authenticate users via public key (from `~/.ssh/id_ed25519`)
- [ ] Launch TUI in SSH session
- [ ] Route messages between SSH sessions

### Phase 2: Message Routing
- [ ] Connect SSH sessions to message queue
- [ ] Handle concurrent SSH users
- [ ] Broadcast encrypted messages to all connected peers

### Phase 3: Advanced Features
- [ ] SSH key management (authorized_keys)
- [ ] User profiles (store X25519 pubkeys)
- [ ] Persistent message queue (optional)
- [ ] Logging & monitoring

## Usage

### Start Relay Server (130.204.65.82)

```bash
# Run with SSH server enabled
globy relay --ssh-port 2222 --http-port 3000

# Output:
# 🔑 SSH Server starting on 0.0.0.0:2222
# 📡 Globy relay running
# ✅ Ready for connections
```

### User Connects (From anywhere)

```bash
# Alice from laptop
ssh alice@130.204.65.82 -p 2222

# Bob from Uganda (home network, no public IP)
ssh bob@130.204.65.82 -p 2222

# Charlie from mobile hotspot
ssh charlie@130.204.65.82 -p 2222

# All see:
# ┌─────────────────────┐
# │ Globy Chat          │
# ├─────────────────────┤
# │ Alice: Hello Bob!   │
# │ Bob: Hi Alice! 👋   │
# │                     │
# │ [Type message...]   │
# └─────────────────────┘
```

## Security Model

### What's Encrypted

✅ **Messages** - E2E encrypted (ChaCha20-Poly1305)  
✅ **SSH channel** - TLS-like encryption (SSH protocol)  
✅ **Signatures** - Prove sender identity (ED25519)  

### What's Visible to Relay

⚠️ **SSH pubkeys** - To authenticate users  
⚠️ **Connection metadata** - Who's online  
⚠️ **Message routing** - From/to hashes  
⚠️ **Ciphertext size** - Not plaintext content  

### Threat Model

| Threat | Protection |
|--------|-----------|
| **Network snooper** | SSH encryption + E2E encryption |
| **Relay operator** | E2E encryption (can't read) |
| **Forged messages** | SSH signatures + MAC verification |
| **Replay attacks** | Nonce + timestamp per message |
| **Identity spoofing** | SSH key authentication |

## Code Structure

### ssh_server.rs
```rust
pub struct GlobySSHServer {
    peer_hash: String,
    message_queue: Arc<Mutex<MessageQueue>>,
}

impl GlobySSHServer {
    pub async fn run(&self, port: u16) -> Result<()> {
        // Accept SSH connections
        // Authenticate users
        // Launch TUI in each session
    }
}
```

### Session Handler
```rust
pub struct SessionHandler {
    username: String,
    message_queue: Arc<Mutex<MessageQueue>>,
}

// Handles one SSH session
// - Authenticates user
// - Runs TUI
// - Routes messages
```

### Main Server
```rust
// In server.rs, add:
let ssh_server = GlobySSHServer::new(
    peer_hash.clone(),
    Arc::new(Mutex::new(message_queue.clone()))
);

// Spawn SSH and HTTP servers together
tokio::spawn(ssh_server.run(2222));
tokio::spawn(http_server.run(3000));
```

## Deployment

### Relay Server (130.204.65.82)

```bash
# Install Globy
cargo install --path .

# Run relay with SSH server
globy relay \
  --ssh-port 2222 \
  --http-port 3000 \
  --host 130.204.65.82

# Systemd service
[Unit]
Description=Globy Relay Server
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/globy relay --ssh-port 2222 --http-port 3000
Restart=always

[Install]
WantedBy=multi-user.target
```

### Client Connection

No installation needed! Just SSH:

```bash
# First time
ssh alice@130.204.65.82 -p 2222

# SSH remembers host key
# Next time, just enter your nickname
# Chat appears instantly
```

## Comparison: Old vs New

### Before (Direct P2P)
```
Alice → HTTP POST → Bob:3001
        (needs public IP, port forward, NAT traversal)
```

### After (SSH Terminal Service)
```
Alice → SSH → Relay:2222 → TUI
        (works from anywhere!)

Bob (Uganda) → SSH → Relay:2222 → TUI
        (home network, no setup!)

Charlie (Mobile) → SSH → Relay:2222 → TUI
        (cellular hotspot!)
```

## Future Enhancements

1. **Web Terminal** - Browser-based SSH client (ttyd)
2. **Mobile Apps** - SSH client bundled with Globy
3. **File Sharing** - SFTP support for encrypted files
4. **Voice Chat** - Audio over encrypted tunnel
5. **TUI Improvements** - Better UI/UX for terminal
6. **Key Management** - Web UI for managing SSH keys
7. **Persistent History** - Optional message archiving
8. **Grouping** - Channels/rooms for multiple users

## Performance Considerations

- **SSH overhead**: ~1KB per connection (minimal)
- **TUI rendering**: Local to SSH session (no bandwidth)
- **Messages**: Only encrypted blobs transmitted
- **Relay capacity**: Can handle thousands of concurrent SSH sessions

## Comparison to Existing Services

| Feature | Globy | Slack | Discord | Matrix |
|---------|-------|-------|---------|--------|
| **E2E Encrypted** | ✅ | ❌ | ❌ | ✅ |
| **Self-hosted** | ✅ | ❌ | ❌ | ✅ |
| **SSH Access** | ✅ | ❌ | ❌ | ❌ |
| **Terminal UI** | ✅ | ❌ | ❌ | ✅ |
| **No Public IP** | ✅ | N/A | N/A | N/A |
| **Zero Setup** | ✅ | ❌ | ❌ | ❌ |

---

## Let's Build It! 🚀

The SSH terminal service model solves every NAT/firewall/public-IP issue at once. Users just:

```bash
ssh you@relay
```

Done. Encrypted. Private. Works everywhere.

No ports to forward. No IP addresses to swap. No NAT traversal voodoo.

Just beautiful, secure, terminal-based encrypted chat.
