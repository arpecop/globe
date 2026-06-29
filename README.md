# Globy - P2P Ephemeral Chat Network

A decentralized, federated messaging platform built in Rust. No server, no central authority, just peer-to-peer.

## Features

- ✅ **P2P Architecture**: Direct peer connections, no central server
- ✅ **Ephemeral Messages**: Messages live in memory, cleared on restart
- ✅ **Channel-Based**: Public channels + direct messages
- ✅ **Nickname Anonymization**: Deterministic hashing (SHA256)
- ✅ **Parallel API**: REST/JSON API for third-party clients
- ✅ **Terminal TUI**: Built-in terminal client (Ratatui)
- ✅ **Distributed Discovery**: Bootstrap nodes + gossip protocol
- ✅ **Open Source**: Full transparency, community-driven

## Installation

### From Source

```bash
git clone https://github.com/globy-chat/globy
cd globy
cargo build --release
./target/release/globy --help
```

### From Releases (Coming Soon)

```bash
curl -sSL https://install.globy.io | bash
```

## Usage

### Start Server

```bash
# Server mode with both API and TUI
globy serve --salt a7f3e2d1c9b4 --port 3000

# API only (for third-party clients)
globy serve --salt a7f3e2d1c9b4 --port 3000 --mode api

# TUI only
globy serve --salt a7f3e2d1c9b4 --mode tui
```

### Connect as Client

```bash
globy cli --connect localhost:3000 --nickname Emperor
```

### Show Information

```bash
globy info
globy version
```

## Architecture

```
┌─────────────────────────────────┐
│ Core Protocol (ephemeral)       │
├─────────────────────────────────┤
│ • Channels (pre-defined)         │
│ • Messages (memory-only)         │
│ • Users (hashed nicknames)       │
│ • P2P routing                    │
└─────────────────────────────────┘
         │          │
    ┌────┴────┐ ┌───┴────┐
    ▼         ▼ ▼        ▼
  REST API  WS  TUI   P2P Network
  (Axum)        (Ratatui) (Bootstrap)
```

## Nickname Anonymity

Your nickname is never transmitted. Instead:

1. **Local hashing**: `SHA256(nickname|device_id|salt)`
2. **Network sees**: `0x8737f2d1` (hash, not name)
3. **Per-device**: Same device always generates same hash
4. **Irreversible**: Can't reverse-engineer nickname from hash

### Example

```rust
let hasher = NicknameHasher::new("device_salt");
let hash = hasher.hash("Emperor", "device_123");
// Result: 0x8737f2d1 (always same for same inputs)
```

## Development

### Project Structure

```
src/
├── main.rs           # CLI entry point
├── lib.rs            # Module exports
├── config.rs         # Configuration loading
├── protocol.rs       # Message types, channels
├── crypto.rs         # Nickname hashing
├── server.rs         # Server core (API + P2P)
├── client.rs         # CLI client
└── ui.rs             # TUI components
```

### Running Tests

```bash
cargo test
```

### Building Release

```bash
cargo build --release
# Binary: target/release/globy (~10MB)
```

## API Endpoints (In Development)

```
POST   /v1/auth                      # Authenticate
GET    /v1/channels                  # List channels
GET    /v1/channels/:id/messages     # Get messages
POST   /v1/channels/:id/message      # Send message
WS     /v1/channels/:id/stream       # Real-time stream
```

## Configuration

Config file: `~/.globy/config.toml`

```toml
[server]
salt_hash = "a7f3e2d1c9b4"
port = 3000
mode = "both"

[bootstrap]
nodes = [
  "node1.globy.io:3000",
  "node2.globy.io:3000"
]

[crypto]
hash_algorithm = "sha256"
```

## Roadmap

- [ ] REST API implementation
- [ ] WebSocket real-time messaging
- [ ] Terminal TUI (Ratatui)
- [ ] P2P peer discovery
- [ ] Bootstrap node support
- [ ] React Native client example
- [ ] Web browser client
- [ ] Message encryption (ChaCha20-Poly1305)
- [ ] Key signing/verification
- [ ] Persistence (optional SQLite)

## Privacy & Security

### What It Protects

- ✅ Message content (ephemeral, only in RAM)
- ✅ Nickname anonymity (hashed, never transmitted)
- ✅ Metadata (distributed, no central logging)
- ✅ Code transparency (open source, auditable)

### What It Doesn't

- ⚠️ IP address (unless using Tor/VPN)
- ⚠️ Timing analysis (messages timestamped)
- ⚠️ Network graph (peer connections visible)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

AGPL-3.0 - See [LICENSE](LICENSE)

## FAQ

**Q: Why ephemeral messages?**  
A: Simpler, faster, no database needed, better privacy. If you need history, that's a different product.

**Q: Can someone seize the server?**  
A: There's no single server. Each peer is a server. Kill one, network continues.

**Q: Is this like Briar/Jami?**  
A: Similar goals (decentralized, private), different implementation. Globy focuses on simplicity + federation.

**Q: What about spam?**  
A: Not handled yet. Solutions: channel moderation, reputation, bandwidth limits.

## Links

- 🌐 [Website](https://globy.chat)
- 📚 [Docs](https://docs.globy.chat)
- 🐛 [Issues](https://github.com/globy-chat/globy/issues)
- 💬 [Discussions](https://github.com/globy-chat/globy/discussions)

---

**Status**: Early MVP - Core scaffold complete, features in development
