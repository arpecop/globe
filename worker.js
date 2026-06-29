/**
 * Globy Handshake Worker
 * Deployed to: globy-handshake.workers.dev
 * Purpose: Peer discovery (not message relay)
 */

// In-memory peer registry
// Format: { channel_name: [{ip, port, timestamp}] }
let peerRegistry = new Map();

// Clean up stale peers every 60 seconds
setInterval(() => {
  const now = Date.now();
  const PEER_TIMEOUT = 300000; // 5 minutes

  for (const [channel, peers] of peerRegistry.entries()) {
    const alive = peers.filter(p => now - p.timestamp < PEER_TIMEOUT);
    if (alive.length === 0) {
      peerRegistry.delete(channel);
    } else {
      peerRegistry.set(channel, alive);
    }
  }
}, 60000);

export default {
  async fetch(request) {
    const url = new URL(request.url);
    const path = url.pathname;
    const method = request.method;

    // POST /register - Peer registers itself
    if (method === 'POST' && path === '/register') {
      try {
        const { channel, ip, port, nickname_hash } = await request.json();

        if (!channel || !ip || !port) {
          return new Response(JSON.stringify({ error: 'Missing required fields' }), {
            status: 400,
            headers: { 'Content-Type': 'application/json' }
          });
        }

        // Add/update peer
        if (!peerRegistry.has(channel)) {
          peerRegistry.set(channel, []);
        }

        const peers = peerRegistry.get(channel);

        // Check if peer already exists, update timestamp
        const existingPeer = peers.find(p => p.ip === ip && p.port === port);
        if (existingPeer) {
          existingPeer.timestamp = Date.now();
          existingPeer.nickname_hash = nickname_hash;
        } else {
          peers.push({
            ip,
            port,
            nickname_hash,
            timestamp: Date.now()
          });
        }

        return new Response(JSON.stringify({
          status: 'registered',
          peer_count: peers.length
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        });
      } catch (err) {
        return new Response(JSON.stringify({ error: err.message }), {
          status: 400,
          headers: { 'Content-Type': 'application/json' }
        });
      }
    }

    // GET /discover/:channel - Get peers for channel
    if (method === 'GET' && path.startsWith('/discover/')) {
      const channel = path.replace('/discover/', '');

      if (!channel) {
        return new Response(JSON.stringify({ error: 'Channel not specified' }), {
          status: 400,
          headers: { 'Content-Type': 'application/json' }
        });
      }

      const peers = peerRegistry.get(channel) || [];

      // Filter out stale peers
      const now = Date.now();
      const PEER_TIMEOUT = 300000;
      const alive = peers.filter(p => now - p.timestamp < PEER_TIMEOUT);

      return new Response(JSON.stringify({
        channel,
        peer_count: alive.length,
        peers: alive.map(p => ({ ip: p.ip, port: p.port }))
      }), {
        status: 200,
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*'
        }
      });
    }

    // GET /channels - List all active channels
    if (method === 'GET' && path === '/channels') {
      const now = Date.now();
      const PEER_TIMEOUT = 300000;

      const channels = Array.from(peerRegistry.entries())
        .map(([name, peers]) => {
          const alive = peers.filter(p => now - p.timestamp < PEER_TIMEOUT);
          return { name, peer_count: alive.length };
        })
        .filter(c => c.peer_count > 0);

      return new Response(JSON.stringify({
        channels,
        total: channels.length
      }), {
        status: 200,
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*'
        }
      });
    }

    // GET /status - Health check
    if (method === 'GET' && path === '/status') {
      return new Response(JSON.stringify({
        status: 'healthy',
        channels: peerRegistry.size,
        timestamp: Date.now()
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      });
    }

    return new Response(JSON.stringify({ error: 'Not found' }), {
      status: 404,
      headers: { 'Content-Type': 'application/json' }
    });
  }
};
