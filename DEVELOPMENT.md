# Development Guide

## Current Status

✅ **MVP Scaffold Complete**
- 674 lines of Rust code
- 3 git commits
- 33 dependencies
- Compiles cleanly with only dead-code warnings

## What's Built

### Core Infrastructure
- ✅ CLI argument parsing (clap)
- ✅ Configuration system (TOML)
- ✅ Protocol data types (Message, Channel, User, Peer)
- ✅ Nickname hashing (SHA256-based, deterministic)
- ✅ Terminal UI skeleton (Ratatui)

### Modules

| Module | Status | Lines | Purpose |
|--------|--------|-------|---------|
| `main.rs` | ✅ | 115 | CLI entry point, command routing |
| `config.rs` | ✅ | 60 | Configuration loading/saving |
| `protocol.rs` | ✅ | 145 | Message/channel/user types |
| `crypto.rs` | ✅ | 60 | Nickname hashing, testing |
| `server.rs` | 🟡 | 50 | Server skeleton, ready for API |
| `client.rs` | ✅ | 65 | TUI event loop, terminal handling |
| `ui.rs` | ✅ | 180 | Ratatui components, layout |

## What Needs to Be Built

### Phase 1: Server API (Priority: HIGH)
**Goal**: Make server a functional HTTP API

**Tasks**:
1. Implement REST API with Axum
   - `POST /v1/auth` - authenticate user
   - `GET /v1/channels` - list channels
   - `GET /v1/channels/:id/messages` - get messages
   - `POST /v1/channels/:id/message` - send message
   - `WS /v1/channels/:id/stream` - real-time updates

2. Add channel state management
   - Store channels in-memory (ephemeral)
   - Track users per channel
   - Relay messages to connected peers

3. Add user management
   - Track active users per channel
   - Track peer hosting info

**Code location**: `src/server.rs`  
**Est. lines**: 400-500  
**Effort**: 1-2 days

### Phase 2: WebSocket Integration (Priority: HIGH)
**Goal**: Real-time messaging between clients

**Tasks**:
1. Implement WebSocket handler in Axum
2. Add message subscription system
3. Broadcast messages to all subscribers
4. Handle peer disconnect/reconnect

**Code location**: `src/server.rs` (new module: `src/websocket.rs`)  
**Est. lines**: 200-300  
**Effort**: 1 day

### Phase 3: P2P Networking (Priority: MEDIUM)
**Goal**: Peer discovery and message relay

**Tasks**:
1. Peer-to-peer connection logic
   - Connect to other peers
   - Exchange peer lists
   - Handle disconnections

2. Message relay
   - Forward messages to all peers
   - Handle duplicates (dedup by message ID)
   - Track which peers have seen message

3. Bootstrap node discovery
   - Query bootstrap nodes for peer list
   - Register self as available peer

**Code location**: `src/server.rs` (new module: `src/p2p.rs`)  
**Est. lines**: 300-400  
**Effort**: 2-3 days

### Phase 4: Enhanced TUI (Priority: LOW)
**Goal**: Better user experience

**Tasks**:
1. Real-time message updates
   - Poll server for new messages
   - Scroll/pagination
   - Message highlighting

2. Better input handling
   - Multi-line messages
   - Command support (/help, /join, /leave)
   - Tab completion

3. Better layout
   - User list
   - Connection status
   - Peer count

**Code location**: `src/client.rs`, `src/ui.rs`  
**Est. lines**: 200-300  
**Effort**: 1-2 days

## Build Instructions

### Local Development

```bash
# Build debug binary
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- serve --salt test --port 3000
```

### Release Build

```bash
# Build single-target release
cargo build --release

# Binary: target/release/globy (~8MB)
```

### Cross-Platform Build

```bash
# Requires cross-rs or manual toolchain setup
./build.sh

# Outputs to releases/ directory
```

## Testing Strategy

### Unit Tests (Current)

```bash
# Test crypto hashing
cargo test crypto::tests

# Output:
# test crypto::tests::test_deterministic_hashing ... ok
# test crypto::tests::test_different_device_different_hash ... ok
# test crypto::tests::test_different_nickname_different_hash ... ok
# test crypto::tests::test_hash_format ... ok
```

### Integration Tests (TODO)

1. Start server
2. Connect client
3. Send message
4. Verify receipt
5. Stop server

### Performance Tests (TODO)

- Message throughput (msgs/sec)
- Memory usage under load
- CPU usage during relay

## Code Style

### Rust Conventions

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Keep functions under 50 lines
- Prefer early returns

### Naming

- `pub struct Message` - User-facing types
- `fn handle_message()` - Event handlers
- `const MAX_MESSAGES: usize` - Constants

### Comments

Only add comments for WHY, not WHAT:

```rust
// ❌ Bad
// Add one to x
let y = x + 1;

// ✅ Good
// Reserve slot for future messages
let capacity = max_messages + 1;
```

## Architecture Decisions

### Why Ephemeral Messages?

1. **Simpler**: No database, no persistence logic
2. **Faster**: Everything in RAM
3. **Safer**: No message history to compromise
4. **Cleaner**: Server restart = clean slate

### Why Nickname Hashing?

1. **Deterministic**: Same nickname = same hash on same device
2. **Irreversible**: Can't crack hash back to nickname
3. **Per-device**: Different devices get different hashes
4. **Transparent**: Algorithm is open-source, can be audited

### Why Bootstrap Nodes?

1. **Discovery**: New peers need to find existing peers
2. **Resilience**: Multiple bootstrap nodes prevent single-point failure
3. **Scalability**: Bootstrap nodes don't need to route messages
4. **Simplicity**: Just serves as a registry

## Deployment

### Single Machine

```bash
globy serve --salt local --port 3000
globy cli --connect localhost:3000
```

### Cloudflare Tunnel

```bash
# Expose server through Cloudflare
# https://developers.cloudflare.com/cloudflare-one/connections/connect-applications/install-and-setup/tunnel-guide/

cloudflare-tunnel run
```

### Docker

```dockerfile
FROM rust:latest
WORKDIR /app
COPY . .
RUN cargo build --release
CMD ["./target/release/globy", "serve", "--salt", "docker", "--port", "3000"]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: globy-node
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: globy
        image: globy:latest
        ports:
        - containerPort: 3000
```

## Monitoring & Logging

### Current Logging

```rust
// Uses tracing crate
tracing::info!("Server started");
tracing::debug!("Message received: {:?}", msg);
tracing::error!("Failed to send: {}", err);
```

### Enable Logging

```bash
RUST_LOG=debug ./target/debug/globy serve --salt test --port 3000
```

### Future: Metrics

- Prometheus scrape endpoint
- Message throughput (msgs/sec)
- Connected peers count
- Memory usage

## Security Considerations

### Input Validation

- [ ] Validate message length (max 10KB)
- [ ] Validate channel names (alphanumeric + dash)
- [ ] Validate nickname length (1-32 chars)

### Rate Limiting

- [ ] Per-peer message rate limit
- [ ] Per-channel join rate limit
- [ ] Per-message routing limit

### Authentication

- [ ] Optional: Key-based authentication
- [ ] Optional: Token rotation
- [ ] Optional: Message signing (ed25519)

## Known Limitations

1. **No message history** - By design. Messages are ephemeral.
2. **No central moderation** - Channels can't be taken down (or created) centrally.
3. **No encryption** - Messages in plaintext (for now).
4. **No spam protection** - No rate limiting or reputation system.
5. **IPv4/IPv6 visible** - Peers can see each other's IPs.

## Future Enhancements

### Short Term (Next Sprint)

- [ ] REST API implementation
- [ ] WebSocket support
- [ ] Basic P2P relay
- [ ] Server persistence (optional SQLite)

### Medium Term

- [ ] End-to-end encryption
- [ ] Message signing
- [ ] Reputation system
- [ ] Moderation tools
- [ ] Web client (React)

### Long Term

- [ ] Mobile apps (iOS/Android)
- [ ] Desktop apps (Electron)
- [ ] Decentralized identity
- [ ] Multi-signature channels
- [ ] Proof-of-work spam prevention

## References

- Rust Book: https://doc.rust-lang.org/book/
- Ratatui: https://docs.rs/ratatui/latest/ratatui/
- Axum: https://docs.rs/axum/latest/axum/
- Tokio: https://tokio.rs/

---

**Last updated**: 2026-06-29  
**Status**: Ready for Phase 1 implementation
