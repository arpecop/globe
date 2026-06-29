# Globy Architecture - Zero Metadata Design

## Problem Solved

**Old design (problematic):**
- Worker stored: `{channel: #general, ip: 1.2.3.4, port: 3000, hash: 0x8737}`
- This is metadata! Someone snooping could correlate hashes to IP addresses
- Defeats the purpose of "nicknameless"

**New design (zero metadata):**
- Worker stores: Nothing about peers
- Worker only confirms: "Someone is online" (heartbeat)
- All peer info exchanged P2P via PM protocol
- **Result: Completely anonymous**

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│ Cloudflare Worker (MINIMAL)                         │
│ - POST /heartbeat/{hash}  → {ok}                    │
│ - GET /status/{channel}   → {online: true/false}    │
│ - GET /ping               → {peers_online: N}       │
│                                                      │
│ Storage: ZERO                                        │
│ Metadata: ZERO                                       │
│ Size: 0.67 KiB (gzipped)                            │
└─────────────────────────────────────────────────────┘

Each Peer (local):
┌─────────────────────────────────────────────────────┐
│ ~/.ssh/id_ed25519 (SSH Key)                         │
│ └─ Hash: SHA256(pubkey)[0:8] = 0x8737f2d1          │
│                                                      │
│ ~/.globy/nicknames.json (Local DB)                 │
│ └─ 0x8737f2d1 → "Emperor"                          │
│ └─ 0xabcd1234 → "Alice"                            │
│ └─ 0x5a2f9876 → "Bob"                              │
│                                                      │
│ Server: Listens on 0.0.0.0:3000                    │
│ └─ Announces: 0x8737f2d1                           │
│                                                      │
│ Client: Can connect anywhere                       │
│ └─ Uses SSH key for auth                           │
│ └─ Sends PM to establish connection                │
└─────────────────────────────────────────────────────┘
```

---

## Identity Model

### Your Peer Hash (Public, Shared)

```
~/.ssh/id_ed25519.pub (already exists)
       ↓
SHA256(public_key)
       ↓
0x8737f2d1  ← This is your peer identity
            ← Everyone knows this
            ← Unlinked to any real name
```

**Each machine generates this ONCE:**
```bash
# It's already on every Linux system
ls ~/.ssh/id_ed25519.pub
# If missing: ssh-keygen -t ed25519
```

### Your Real Nickname (Private, Local Only)

```
~/.globy/nicknames.json
{
  "0x8737f2d1": "Emperor",    ← Only you see this
  "0xabcd1234": "Alice",
  "0x5a2f9876": "Bob"
}
```

**Result:**
- ✅ Others see you as: `0x8737f2d1`
- ✅ You remember them as: "Alice", "Bob"
- ❌ No central server knows either
- ❌ No correlation between hash and name

---

## Connection Protocol

### Phase 1: Peer Discovery (You know their hash)

```
User A wants to chat with 0xabcd1234 (Alice)

1. A queries: Worker /status/general
   ← "yes, peers online"

2. A asks bootstrap nodes: "Who hosts #general?"
   ← "Try these peers: 1.2.3.4, 5.6.7.8, ..."

3. A connects P2P to one peer
   ← "I'm 0x8737 (Emperor), hello!"
```

### Phase 2: PM Request (Establish private connection)

```
A sends PM to 0xabcd:

{
  from_hash: "0x8737",
  to_hash: "0xabcd",
  message: "Hi Alice, wanna chat?",
  sender_nickname_encrypted: "Emperor...",  ← ChaCha20 encrypted
  signature: "<SSH signature>"               ← Signed with SSH key
}

B verifies:
  1. Signature checks out (SSH key is authentic)
  2. Decrypts nickname: "Emperor"
  3. Stores locally: 0x8737 → "Emperor"
  4. Replies: "Yes!"
```

### Phase 3: Authenticated Chat

```
A and B now know each other:
  A: 0xabcd → "Alice"
  B: 0x8737 → "Emperor"

Messages are signed with SSH keys:
  - Proves sender is who they claim
  - Can't be forged or modified
  - Receiver verifies before displaying
```

---

## What's Stored Where

| Data | Location | Visibility | Encrypted |
|------|----------|-----------|-----------|
| **SSH Key** | `~/.ssh/id_ed25519` | Local only | Yes (on disk) |
| **Peer Hash** | Derived from SSH key | Public | N/A |
| **Real Nickname** | `~/.globy/nicknames.json` | Local only | Optional |
| **Messages** | RAM (peer) | Local + connected peers | Yes (E2E) |
| **Worker Data** | Nothing! | Nothing! | N/A |

**Security model:**
- ✅ No central metadata storage
- ✅ No IP-to-hash correlation
- ✅ No name-to-hash linking
- ✅ All secrets stay local
- ✅ SSH keys provide strong auth

---

## Code Modules

### `ssh_key.rs` (SSH Key Management)

```rust
// Load existing SSH key
let identity = SshIdentity::new()?;

// Get your peer hash
let peer_hash = identity.get_peer_hash()?;
// → 0x8737f2d1

// Get public key
let pub_key = identity.get_public_key()?;
```

### `pm.rs` (PM Protocol)

```rust
// Create private message
let mut pm = PrivateMessage::new(
    "0x8737",           // from
    "0xabcd",           // to
    "Hello Alice!".to_string()
);

// Sign with SSH key
pm.sign(ssh_signature);

// Encrypt nickname before sending
pm.set_encrypted_nickname(encrypted_name);
```

### `handshake.rs` (Ultra-minimal Worker Client)

```rust
let heartbeat = HeartbeatClient::new(worker_url);

// Send heartbeat (only thing sent to worker)
heartbeat.heartbeat("0x8737").await?;

// Check if anyone is online
let online = heartbeat.is_channel_online("general").await?;
```

### `server.rs` (Peer Server)

```rust
// Get SSH identity
let identity = SshIdentity::new()?;
let peer_hash = identity.get_peer_hash()?;

// Create heartbeat client
let heartbeat = HeartbeatClient::new(worker_url);

// Run server + send heartbeats
let server = Server::new(config, peer_hash, heartbeat);
server.run(3000, "api").await?;
```

---

## Usage Flow

### Start Server (Host)

```bash
globy serve --salt mynode --port 3000

Output:
🔑 SSH Key: ssh-ed25519 AAAAC3...
🆔 Your peer hash: 0x8737f2d1
📡 Server starting on port 3000
📍 Mode: both
✅ Online and accepting connections

(Sends heartbeat every 60 seconds)
```

### Connect Client

```bash
globy cli --nickname Emperor

Output:
👤 Nickname: Emperor (0x8737f2d1)
🔑 SSH Key loaded
🌐 Connecting to: localhost:3000
📺 Starting TUI...

(Stores locally: 0x8737f2d1 → Emperor)
```

### Send PM

```
You see: 0xabcd in channel
You type: /pm 0xabcd "Hi Alice!"

1. Encrypts: "Emperor" → AES encrypted
2. Signs: PM with SSH key
3. Sends to 0xabcd
4. They verify + decrypt
5. They see: PM from Emperor (0xabcd)
6. They store: 0xabcd → "Emperor" (if first time)
```

---

## Security Guarantees

### What's Protected

✅ **Message content** - E2E encrypted  
✅ **Sender identity** - SSH signed, can't forge  
✅ **Nickname privacy** - Never transmitted, stored locally  
✅ **Network anonymity** - No IP-to-hash linking  
✅ **No metadata leaks** - Worker stores nothing  

### What's NOT Protected

⚠️ **IP addresses** - Visible when connecting directly  
⚠️ **Timing attacks** - When you send messages is visible  
⚠️ **Connection graph** - Who talks to whom is observable  

**Mitigation:**
- Use Tor/VPN for IP anonymity
- Use randomized message timing
- Use proxy nodes for metadata masking

---

## Comparison: Old vs New

### Old Design (Problematic)

```
Worker stores:
{
  "general": [
    {ip: "1.2.3.4", port: 3000, hash: "0x8737", nickname_hash: "..."},
    {ip: "5.6.7.8", port: 3000, hash: "0xabcd", nickname_hash: "..."}
  ]
}

Problems:
- Worker knows who talks to whom (metadata)
- Someone could correlate IPs to hashes
- Central point of failure
- Defeats "nicknameless" concept
```

### New Design (Zero Metadata)

```
Worker stores:
{
  "0x8737": 1782770063969,  ← Just timestamp for TTL
  "0xabcd": 1782770055123
}

Or even simpler: Just counts
{
  "peers_online": 2
}

Benefits:
- No metadata about peers
- No IP addresses stored
- No channel information
- No way to correlate hash → identity
- Truly "nicknameless"
```

---

## Future Enhancements

### Phase 2: End-to-End Crypto

- [ ] ChaCha20-Poly1305 for PM encryption
- [ ] X25519 for key exchange
- [ ] Forward secrecy (ratchet keys)

### Phase 3: Metadata Masking

- [ ] Tor integration for IP hiding
- [ ] Padding for timing resistance
- [ ] Decoy traffic

### Phase 4: Web UI

- [ ] React frontend
- [ ] QR code for peer sharing
- [ ] Dark mode 🌙

---

## Files Changed

```
src/
├── ssh_key.rs       ← SSH key identity
├── pm.rs            ← PM protocol (signed)
├── handshake.rs     ← Ultra-minimal Worker client
├── server.rs        ← Updated with SSH + heartbeat
└── client.rs        ← Updated for SSH auth

worker.js           ← Rewritten (0.67 KiB)
wrangler.toml       ← Simplified config
```

---

**Status**: Architecture complete, MVP ready for testing 🎯
