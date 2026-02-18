const CACHE_NAME = '3dt-cache-v2';

// Network-first for all assets (always get latest, cache as offline fallback)
const CACHEABLE_EXTENSIONS = ['.wasm', '.js', '.html', '.css', '.glb', '.png', '.jpg', '.json'];

self.addEventListener('install', (event) => {
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((names) =>
      Promise.all(
        names
          .filter((name) => name !== CACHE_NAME)
          .map((name) => caches.delete(name))
      )
    )
  );
  self.clients.claim();
});

self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);

  // Skip non-GET requests and WebSocket
  if (event.request.method !== 'GET') return;
  if (url.protocol === 'ws:' || url.protocol === 'wss:') return;

  const isCacheable = CACHEABLE_EXTENSIONS.some((ext) => url.pathname.endsWith(ext))
    || url.pathname === '/';

  if (isCacheable) {
    // Network-first: try network, fall back to cache (for offline)
    event.respondWith(
      fetch(event.request)
        .then((response) => {
          if (response.ok) {
            const clone = response.clone();
            caches.open(CACHE_NAME).then((cache) => cache.put(event.request, clone));
          }
          return response;
        })
        .catch(() => caches.match(event.request))
    );
  }
});
