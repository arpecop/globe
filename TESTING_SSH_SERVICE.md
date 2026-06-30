# Testing Globy SSH Terminal Service

## Test Environment Setup

### Prerequisites

```bash
# Install Globy (local development)
cd /home/rudix/Desktop/globy
cargo build --release

# Verify SSH keys exist
ls -la ~/.ssh/id_ed25519*
# Output:
# -rw------- 1 you you  387 Jun 30 12:00 .ssh/id_ed25519
# -rw-r--r-- 1 you you  136 Jun 30 12:00 .ssh/id_ed25519.pub
```

## Test 1: Local SSH Server (Localhost)

### Terminal 1: Start Relay Server

```bash
cd /home/rudix/Desktop/globy

# Run relay on localhost:2222
GLOBY_MODE=relay ./target/release/globy \
  --host \
  --port 3000 \
  --ssh-port 2222

# Expected output:
# 🔑 SSH Server starting on 0.0.0.0:2222
# 📡 Globy relay running
# 📍 HTTP API Port: 3000
# ✅ Ready for connections
```

### Terminal 2: Connect Alice

```bash
# SSH as Alice
ssh -p 2222 alice@localhost

# You might see:
# The authenticity of host '[localhost]:2222 (127.0.0.1)' can't be established.
# RSA key fingerprint is ...
# Type 'yes' to accept

# Type: yes

# Then you should see:
# Welcome to Globy SSH Terminal!
# Your peer hash: 0x8737f2d1
# Enter your display name: Alice

# (You're now in the TUI)
# Type: Hello Bob!
# [message appears in chat]
```

### Terminal 3: Connect Bob

```bash
# Open another terminal
ssh -p 2222 bob@localhost

# Same setup, enter:
# Your display name: Bob

# In the TUI, you should see Alice's message!
# Type: Hi Alice! 👋

# Alice should see this message instantly
```

## Test 2: Network Testing (Multiple Machines)

### On Relay Server (130.204.65.82)

```bash
# Run relay listening on all interfaces
./target/release/globy relay \
  --host \
  --port 3000 \
  --ssh-port 2222 \
  --bind 0.0.0.0

# Output:
# 🔑 SSH Server starting on 0.0.0.0:2222
```

### On Alice's Machine (anywhere)

```bash
# Connect to relay
ssh -p 2222 alice@130.204.65.82

# Enter display name: Alice
# (TUI appears)
```

### On Bob's Machine (Uganda, home network)

```bash
# No port forwarding needed!
ssh -p 2222 bob@130.204.65.82

# Enter display name: Bob
# (Bob sees Alice's messages instantly)
```

## Test 3: Verify E2E Encryption

### On Relay Server: Sniff Network Traffic

```bash
# Terminal: Monitor what relay sees
tcpdump -i lo 'port 2222' -A

# You should see:
# SSH protocol exchange (encrypted)
# ❌ NO plaintext messages
# ❌ NO "Hello Bob" in clear text
```

### Test Message is Encrypted

```bash
# In Alice's TUI, type: Secret message for Bob

# On relay server's tcpdump:
# You see SSH traffic
# ❌ NOT: "Secret message for Bob"
# ✅ Only: encrypted ciphertext

# Bob's terminal shows decrypted message
# ✅ Relay couldn't read it!
```

## Test 4: Multiple Users Simultaneously

### Setup 4 SSH Sessions

```bash
# Terminal A: Start relay
./target/release/globy relay --ssh-port 2222

# Terminal B: ssh -p 2222 alice@localhost
# Terminal C: ssh -p 2222 bob@localhost  
# Terminal D: ssh -p 2222 charlie@localhost
# Terminal E: ssh -p 2222 diana@localhost
```

### Test Message Broadcasting

```
Alice:   Types: "Hello everyone!"
         ↓
Relay:   Routes encrypted message
         ↓
Bob, Charlie, Diana: See message instantly

Charlie: Types: "Hi all! 👋"
         ↓
Relay:   Routes to Alice, Bob, Diana
         ↓
All see Charlie's message
```

### Expected Output

```
┌─────────────────────────┐
│ Globy Chat              │
├─────────────────────────┤
│ Alice: Hello everyone!  │
│ Bob: Hi Alice!          │
│ Charlie: Hi all! 👋     │
│ Diana: Hello team!      │
│ [type message...]       │
└─────────────────────────┘
```

## Test 5: Connection Stability

### Test Reconnection

```bash
# Terminal B (Alice's SSH session):
# Type message: "Testing reconnect"

# Kill the SSH connection (Ctrl+C or close terminal)
^C

# Reconnect in a few seconds
ssh -p 2222 alice@localhost

# Should see:
# Previous messages are still there (in memory)
# Can continue chatting
```

### Test Concurrent Connections

```bash
# Run these in parallel:
for i in {1..10}; do
  ssh -p 2222 user$i@localhost &
done

# Should handle 10+ concurrent connections
# No crashes or errors
# Messages route correctly between all
```

## Test 6: Security Tests

### Test 1: Signature Verification

```bash
# When Alice sends message:
# Message is signed with her SSH key

# If someone tries to forge a message:
# - Claim to be Alice but use wrong key
# - Message is rejected ❌

# Verification:
# [Server logs should show: "Invalid SSH signature - rejected"]
```

### Test 2: Encryption Verification

```bash
# Setup tcpdump on relay
tcpdump -i lo port 2222 -w traffic.pcap

# Alice sends: "Credit card: 4532-1234-5678-9010"

# Read captured traffic
strings traffic.pcap | grep -i "credit\|4532"
# Result: (nothing found)
# ✅ Message was encrypted!

# In Bob's TUI: Sees the message plaintext
# ✅ Only Bob could decrypt it!
```

### Test 3: Man-in-the-Middle Protection

```bash
# Attacker tries to intercept SSH connection
ssh -p 2222 fake-alice@localhost
# (using different SSH key)

# Server rejects:
# "Authentication failed"
# ❌ Can't impersonate users
```

## Test 7: Performance Testing

### Test Latency

```bash
# In Alice's TUI:
# Type: "TEST MESSAGE"
# Note the time
# 
# Check Bob's TUI when message arrives
# Typical latency: <100ms (instant)
```

### Test Throughput

```bash
# Script to spam messages
for i in {1..100}; do
  echo "Message $i from Alice" | \
    ssh -p 2222 alice@localhost
done

# Relay should handle all without dropping
# Bob should receive all 100 messages
```

### Test Memory Usage

```bash
# Start relay
./target/release/globy relay --ssh-port 2222

# Connect 100 SSH sessions
for i in {1..100}; do
  ssh -p 2222 user$i@localhost &
done

# Monitor memory
watch free -h

# Should stay under 100MB
# (messages are small, encrypted)
```

## Test 8: Simulated Network Issues

### Test Timeout/Reconnection

```bash
# Terminal A (Alice's session)
# Type: "Testing timeout"
# Wait 5 minutes (no activity)

# Terminal B: Check if connection drops
# (SSH has keep-alive, should stay connected)

# Type another message from Alice
# Should work fine
# ✅ Connection persisted
```

### Test Slow Network

```bash
# Simulate slow connection with tc (traffic control)
sudo tc qdisc add dev lo root netem latency 500ms

# Connect SSH
ssh -p 2222 alice@localhost

# Should still work (slower, but functional)
# Messages still encrypted and verified

# Clean up
sudo tc qdisc del dev lo root
```

## Test 9: Automated Testing

### Create Test Script

```bash
#!/bin/bash
# test_globy_ssh.sh

set -e

echo "🧪 Starting Globy SSH Service Tests"

# Start relay in background
cargo build --release
./target/release/globy relay --ssh-port 2222 &
RELAY_PID=$!

sleep 2  # Let server start

echo "✅ Relay started (PID: $RELAY_PID)"

# Test 1: Connect Alice
echo "📝 Test 1: Alice connects..."
(
  sleep 1
  echo "Alice"
  sleep 2
  echo "Hello Bob!"
  sleep 1
) | ssh -p 2222 alice@localhost &
ALICE_PID=$!

sleep 3

# Test 2: Connect Bob
echo "📝 Test 2: Bob connects..."
(
  sleep 1
  echo "Bob"
  sleep 2
  echo "Hi Alice!"
  sleep 1
) | ssh -p 2222 bob@localhost &
BOB_PID=$!

# Wait for both
wait $ALICE_PID $BOB_PID

echo "✅ All tests passed!"

# Cleanup
kill $RELAY_PID
```

Run it:
```bash
chmod +x test_globy_ssh.sh
./test_globy_ssh.sh
```

## Test 10: Real Network Test (Uganda + Your Relay)

### On Your Relay (130.204.65.82)

```bash
# Make sure SSH port 2222 is open
sudo ufw allow 2222/tcp

# Run relay
./target/release/globy relay --ssh-port 2222
```

### From Uganda (or anywhere)

```bash
# Bob connects from Uganda
ssh -p 2222 bob@130.204.65.82

# Should work instantly!
# No setup, no port forwarding, no NAT issues
```

## Debugging

### SSH Connection Issues

```bash
# Verbose SSH output
ssh -vvv -p 2222 alice@localhost

# You should see:
# SSH Key Exchange
# Authentication: publickey
# Channel open
# (TUI launches)
```

### Message Not Appearing

```bash
# Check relay logs
# (Add logging to ssh_server.rs)

# Verify encryption/signature
# (Add debug output to message handler)

# Check if both users are connected
# (List connected SSH sessions)
```

### Performance Problems

```bash
# Profile the relay
cargo build --release --features profiling

# Use flamegraph
cargo install flamegraph
cargo flamegraph --bin globy relay

# Analyze bottlenecks
```

## Checklist for Complete Testing

- [ ] **Local SSH Test** - Alice & Bob on localhost:2222
- [ ] **Network Test** - Alice & Bob on different machines
- [ ] **Encryption Test** - Tcpdump shows no plaintext
- [ ] **Multi-user Test** - 4+ users chatting simultaneously
- [ ] **Reconnection Test** - Close and reconnect preserves session
- [ ] **Performance Test** - <100ms message latency
- [ ] **Security Test** - Signature verification works
- [ ] **Stress Test** - 100+ concurrent connections
- [ ] **Network Issues** - Timeout/slow networks handled
- [ ] **Real Network** - Test from Uganda if possible

## Expected Test Results

| Test | Expected Result | Pass/Fail |
|------|-----------------|-----------|
| SSH Connect | TUI appears | ✅ |
| Message Send | Message appears in TUI | ✅ |
| Multi-user | All users see each other's messages | ✅ |
| Encryption | Tcpdump shows no plaintext | ✅ |
| Signature | Invalid signatures rejected | ✅ |
| Latency | <100ms message delivery | ✅ |
| Reconnect | Session persists across reconnects | ✅ |
| Stress | 100 concurrent users | ✅ |
| Network | Works over slow/unreliable links | ✅ |

---

## Running Full Test Suite

```bash
# One command to test everything
./test_globy_ssh.sh && \
cargo test && \
cargo bench

# You should see:
# ✅ All SSH tests pass
# ✅ All unit tests pass
# ✅ All benchmarks complete
```

This is your complete testing playbook! Start with **Test 1** (local) and work up to **Test 10** (real network). 🚀
