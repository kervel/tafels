const CACHE_NAME = '3dt-cache-v1';

// Cache-first for static assets
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

  // Check if this is a cacheable static asset
  const isCacheable = CACHEABLE_EXTENSIONS.some((ext) => url.pathname.endsWith(ext))
    || url.pathname === '/';

  if (isCacheable) {
    event.respondWith(
      caches.match(event.request).then((cached) => {
        // Return cached version, but also update cache in background
        const fetchPromise = fetch(event.request)
          .then((response) => {
            if (response.ok) {
              const clone = response.clone();
              caches.open(CACHE_NAME).then((cache) => cache.put(event.request, clone));
            }
            return response;
          })
          .catch(() => cached);

        return cached || fetchPromise;
      })
    );
  }
});
