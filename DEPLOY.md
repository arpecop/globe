# Deploying Globy Handshake Worker

The Globy handshake server is a **Cloudflare Worker** that serves as a peer registry for P2P discovery.

## Prerequisites

1. **Cloudflare Account** (free tier works)
   - Sign up: https://dash.cloudflare.com/

2. **Wrangler CLI**
   ```bash
   npm install -g wrangler
   ```

3. **Node.js** (v16+)

## Deployment Steps

### 1. Authenticate with Cloudflare

```bash
wrangler login
# Opens browser, authorize access to your account
```

### 2. Deploy Worker

```bash
cd /home/rudix/Desktop/globy
wrangler deploy --name globy-handshake
```

Expected output:
```
✅ Uploaded globy-handshake
📝 Your worker is live at:
   https://globy-handshake.workers.dev
```

### 3. Test Handshake Server

```bash
# Check health
curl https://globy-handshake.workers.dev/status

# Response:
# {"status":"healthy","channels":0,"timestamp":1234567890}
```

---

## API Endpoints

### POST /register
Register your peer as hosting a channel

```bash
curl -X POST https://globy-handshake.workers.dev/register \
  -H "Content-Type: application/json" \
  -d '{
    "channel": "general",
    "ip": "1.2.3.4",
    "port": 3000,
    "nickname_hash": "0x8737f2d1"
  }'

# Response:
# {"status":"registered","peer_count":5}
```

### GET /discover/:channel
Get peers currently hosting a channel

```bash
curl https://globy-handshake.workers.dev/discover/general

# Response:
# {
#   "channel": "general",
#   "peer_count": 3,
#   "peers": [
#     {"ip": "1.2.3.4", "port": 3000},
#     {"ip": "5.6.7.8", "port": 3000},
#     {"ip": "9.10.11.12", "port": 3000}
#   ]
# }
```

### GET /channels
List all active channels

```bash
curl https://globy-handshake.workers.dev/channels

# Response:
# {
#   "channels": [
#     {"name": "general", "peer_count": 5},
#     {"name": "dev", "peer_count": 2},
#     {"name": "random", "peer_count": 8}
#   ],
#   "total": 3
# }
```

### GET /status
Health check

```bash
curl https://globy-handshake.workers.dev/status

# Response:
# {"status":"healthy","channels":3,"timestamp":1234567890}
```

---

## How It Works

### 1. Peer Registration (Automatic)

When you run a Globy server:

```bash
globy serve --salt node1 --port 3000 --register general
```

It automatically:
1. Registers itself with the worker
2. Sends heartbeat every 60 seconds
3. Unregisters on shutdown

### 2. Peer Discovery (Automatic)

When you connect as a client:

```bash
globy cli --nickname Emperor
```

It automatically:
1. Queries: `globy-handshake.workers.dev/channels`
2. Lists available channels
3. Queries: `globy-handshake.workers.dev/discover/general`
4. Gets peer list: [1.2.3.4:3000, 5.6.7.8:3000, ...]
5. Connects P2P to random peer
6. Done!

---

## Architecture

```
┌─────────────────────────────────────────────┐
│ Cloudflare Worker (Stateless)               │
│ globy-handshake.workers.dev                 │
├─────────────────────────────────────────────┤
│ • In-memory peer registry (5min TTL)         │
│ • /register - add peer                       │
│ • /discover/:channel - get peers             │
│ • /channels - list all channels              │
│ • Replicated to 200+ data centers            │
└─────────────────────────────────────────────┘
         ↑              ↓
    ┌────┴──────────────┴────┐
    ▼                        ▼
  User A                   User B
  (1.2.3.4:3000)          (5.6.7.8:3000)
  
  Handshake: query worker for peers
  Chat: direct P2P connection
```

---

## Key Features

✅ **Stateless** - No persistent storage needed  
✅ **Distributed** - Runs on Cloudflare's 200+ data centers  
✅ **Fast** - <50ms response time  
✅ **Free** - Cloudflare Workers free tier  
✅ **Scalable** - Handles unlimited channels/peers  
✅ **Resilient** - Can't be seized (it's on Cloudflare infra)  
✅ **Simple** - ~100 lines of code  

---

## Monitoring

### Check Worker Analytics

```bash
wrangler tail globy-handshake
```

Shows real-time requests to your worker.

### View Deployment History

```bash
wrangler deployments list
```

---

## Updating Worker

To update the worker code:

```bash
# Edit worker.js
nano worker.js

# Redeploy
wrangler deploy --name globy-handshake
```

Changes are live immediately (no downtime).

---

## Troubleshooting

### Worker not responding

```bash
# Check status
curl https://globy-handshake.workers.dev/status

# If down, redeploy:
wrangler deploy --name globy-handshake
```

### No peers found

```bash
# Check channels
curl https://globy-handshake.workers.dev/channels

# Check specific channel
curl https://globy-handshake.workers.dev/discover/general
```

### CORS errors

The worker includes `Access-Control-Allow-Origin: *` headers for browser clients.

---

## Production Considerations

### Custom Domain

```toml
# wrangler.toml
route = "handshake.globy.chat/*"
zone_id = "your_zone_id"
```

### Rate Limiting

Add to worker.js if needed:

```javascript
const rate_limit = new Map();

function checkRateLimit(ip) {
  const now = Date.now();
  const window = 60000; // 1 minute
  
  if (!rate_limit.has(ip)) {
    rate_limit.set(ip, []);
  }
  
  const times = rate_limit.get(ip);
  const recent = times.filter(t => now - t < window);
  
  if (recent.length > 100) {
    return false; // Rate limited
  }
  
  recent.push(now);
  rate_limit.set(ip, recent);
  return true;
}
```

### Logging

Enable logging:

```bash
wrangler tail --format pretty
```

---

## Cost

- **Free tier**: 100,000 requests/day (usually enough)
- **Paid tier**: $0.50/million requests after free tier

For Globy, most cost is from peer registrations (every 60 seconds).

With 1000 peers: ~1.4M requests/month = ~$0.50/month

---

**Deployed at**: https://globy-handshake.workers.dev
