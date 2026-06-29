# Globy Quick Start Guide

Welcome to Globy! This guide gets you up and running in 5 minutes.

## Installation

### Option 1: From Source (Recommended for Development)

```bash
git clone https://github.com/globy-chat/globy
cd globy
cargo build --release
./target/release/globy --help
```

### Option 2: Download Binary (Coming Soon)

```bash
curl -sSL https://install.globy.io | bash
```

## Quick Test (Single Machine)

### Terminal 1: Start Server

```bash
./target/debug/globy serve --salt mydevice --port 3000 --mode api
```

Output should show:
```
🚀 Starting Globy Server
📡 Port: 3000
🔐 Mode: api
🔑 Salt: mydevice
```

### Terminal 2: Connect Client

```bash
./target/debug/globy cli --connect localhost:3000 --nickname Emperor
```

You should see the terminal UI with:
- Left panel: Channel list (#general, #dev, #random)
- Center: Message display & input
- Right: User info

## Testing the TUI

### Keyboard Controls

- `Up/Down`: Navigate channels
- `Type`: Enter message
- `Enter`: Send message
- `Esc`: Quit

### Try It

1. Type a message: `hello everyone`
2. Press `Enter` → message appears in chat
3. Type more messages
4. Press `Esc` to exit

## Project Structure

```
globy/
├── src/
│   ├── main.rs          # CLI entry point (commands: serve, cli, version, info)
│   ├── lib.rs           # Module exports
│   ├── config.rs        # Configuration management
│   ├── protocol.rs      # Message types, channels, users
│   ├── crypto.rs        # Nickname hashing (SHA256)
│   ├── server.rs        # Server core (API + P2P relay)
│   ├── client.rs        # CLI client with Ratatui TUI
│   └── ui.rs            # Terminal UI components
├── Cargo.toml           # Dependencies
├── README.md            # Full documentation
├── QUICKSTART.md        # This file
├── install.sh           # Installation script
├── build.sh             # Cross-platform build script
└── .gitignore           # Git ignore rules
```

## What's Working

✅ **CLI argument parsing** - `serve`, `cli`, `version`, `info` commands  
✅ **Configuration system** - Load/save config files  
✅ **Nickname hashing** - Deterministic SHA256-based anonymization  
✅ **Protocol types** - Message, Channel, User, Peer structures  
✅ **Terminal UI** - Ratatui-based interface with channels and messages  
✅ **Demo messages** - Pre-populated chat for testing  
✅ **Channel navigation** - Arrow keys to switch channels  

## What's Next (To Implement)

⏳ **Server API** - REST endpoints (POST /auth, GET /channels, etc.)  
⏳ **WebSocket support** - Real-time message streaming  
⏳ **P2P networking** - Connect to other peers  
⏳ **Message relay** - Forward messages between peers  
⏳ **Bootstrap discovery** - Find peers in the network  
⏳ **Persistence** - Optional SQLite for message history  

## Testing Nickname Hashing

```bash
# Test the hashing function
cargo test --lib crypto
```

Output:
```
test crypto::tests::test_deterministic_hashing ... ok
test crypto::tests::test_different_device_different_hash ... ok
test crypto::tests::test_different_nickname_different_hash ... ok
test crypto::tests::test_hash_format ... ok
```

## Directory Structure

```
/home/rudix/Desktop/globy/
├── target/
│   └── debug/
│       └── globy          # Debug binary (~50MB)
├── .git/                  # Git repository
├── src/                   # Source code
├── Cargo.toml             # Rust dependencies
└── README.md              # Documentation
```

## Troubleshooting

### "Port already in use"

```bash
# Use a different port
./target/debug/globy serve --salt mydevice --port 4000
```

### "Binary not found"

```bash
# Make sure you're in the globy directory
cd /home/rudix/Desktop/globy

# Rebuild
cargo build

# Run
./target/debug/globy --help
```

### "Terminal looks weird"

The TUI works best on:
- macOS Terminal / iTerm2
- Linux: GNOME Terminal, Konsole, Alacritty
- Windows: Windows Terminal (not CMD)

## Next Steps

1. **Explore the code**: Start with `src/main.rs`
2. **Read protocol.rs**: Understand message types
3. **Check crypto.rs**: See how nickname hashing works
4. **Run tests**: `cargo test`
5. **Build release**: `./build.sh`

## Development Tips

### Enable verbose logging

```bash
./target/debug/globy -v serve --salt test --port 3000
```

### Watch files (requires cargo-watch)

```bash
cargo install cargo-watch
cargo watch -x build
```

### Run tests continuously

```bash
cargo watch -x test
```

## Questions?

- 📖 Read [README.md](README.md) for full documentation
- 🐛 Report issues: [GitHub Issues](https://github.com/globy-chat/globy/issues)
- 💬 Discuss: [GitHub Discussions](https://github.com/globy-chat/globy/discussions)

---

**Status**: MVP Scaffold Complete 🎉  
**Next Phase**: Implement server API and P2P networking
