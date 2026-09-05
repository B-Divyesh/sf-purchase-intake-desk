const CACHE = 'intake-desk:demo-shell:__BUILD_SHA__';
const CORE = ['/', '/index.html', '/favicon.svg', '/apple-touch-icon.png', '/og-image.svg', '/manifest.webmanifest'];

self.addEventListener('install', (event) => {
  event.waitUntil(caches.open(CACHE).then((cache) => cache.addAll(CORE)).then(() => self.skipWaiting()));
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys()
      .then((names) => Promise.all(names.filter((name) => name.startsWith('intake-desk:demo-shell:') && name !== CACHE).map((name) => caches.delete(name))))
      .then(() => self.clients.claim()),
  );
});

self.addEventListener('message', (event) => {
  if (event.data?.type !== 'CACHE_URLS' || !Array.isArray(event.data.urls)) return;
  const urls = event.data.urls.filter((url) => typeof url === 'string' && url.startsWith('/') && !url.startsWith('/api/'));
  event.waitUntil(caches.open(CACHE).then((cache) => cache.addAll(urls)).then(() => event.ports[0]?.postMessage('cached')));
});

self.addEventListener('fetch', (event) => {
  const request = event.request;
  const url = new URL(request.url);
  if (request.method !== 'GET' || url.origin !== self.location.origin || url.pathname === '/health' || url.pathname === '/ready' || url.pathname.startsWith('/api/') || request.headers.has('authorization')) return;
  if (request.mode === 'navigate') {
    event.respondWith(fetch(request).catch(async () => {
      const knownRoute = url.pathname === '/' || url.pathname === '/demo' || url.pathname === '/start' || url.pathname === '/app' || url.pathname === '/privacy' || url.pathname === '/terms' || url.pathname === '/auth/callback' || url.pathname.startsWith('/demo/') || url.pathname.startsWith('/app/');
      if (!knownRoute) return new Response('This page is unavailable offline.', { status: 404, headers: { 'Content-Type': 'text/plain; charset=utf-8' } });
      return (await caches.match('/index.html')) || Response.error();
    }));
    return;
  }
  event.respondWith(
    caches.match(request, { ignoreVary: true }).then((cached) => cached || fetch(request).then((response) => {
      if (response.ok) caches.open(CACHE).then((cache) => cache.put(request, response.clone()));
      return response;
    })),
  );
});
