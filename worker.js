/**
 * Globy Worker (Ultra-Minimal)
 *
 * This worker does ONE thing only: heartbeat pings.
 * It stores ZERO metadata, ZERO nicknames, ZERO IPs.
 *
 * Real peer discovery happens P2P via bootstrap nodes.
 * Worker just confirms "someone is online".
 */

let heartbeats = new Map(); // {hash: timestamp}
const HEARTBEAT_TIMEOUT = 300000; // 5 minutes

// Clean up stale heartbeats
function cleanup() {
  const now = Date.now();
  for (const [hash, ts] of heartbeats.entries()) {
    if (now - ts > HEARTBEAT_TIMEOUT) {
      heartbeats.delete(hash);
    }
  }
}

export default {
  async fetch(request) {
    cleanup();

    const url = new URL(request.url);
    const path = url.pathname;

    // POST /heartbeat/:hash - Peer announces itself
    // NO metadata, NO IP, NO channel info
    if (path.startsWith('/heartbeat/')) {
      const hash = path.split('/')[2];
      if (!hash) {
        return json({ error: 'hash required' }, 400);
      }
      heartbeats.set(hash, Date.now());
      return json({ ok: true });
    }

    // GET /status/:channel - Is anyone online in this channel?
    // Returns: {online: true/false} only
    if (path.startsWith('/status/')) {
      return json({ online: heartbeats.size > 0 });
    }

    // GET /ping - Health check
    if (path === '/ping') {
      return json({
        status: 'healthy',
        peers_online: heartbeats.size,
        timestamp: Date.now()
      });
    }

    return json({ error: 'not found' }, 404);
  }
};

// Helper
function json(data, status = 200) {
  return new Response(JSON.stringify(data), {
    status,
    headers: {
      'Content-Type': 'application/json',
      'Access-Control-Allow-Origin': '*'
    }
  });
}
