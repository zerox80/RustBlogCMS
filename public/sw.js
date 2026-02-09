const CACHE_NAME = 'rust-blog-v1';

const urlsToCache = [
  '/',
  '/index.html',
  '/linux-icon.svg',
  '/manifest.json',
];

self.addEventListener('install', (event) => {
  console.log('[SW] Installing service worker - caching core assets');

  event.waitUntil(
    caches.open(CACHE_NAME)
      .then((cache) => {
        console.log(`[SW] Cache opened: ${CACHE_NAME}`);
        return cache.addAll(urlsToCache);
      })
      .then(() => {
        console.log('[SW] Core assets cached successfully');
        return self.skipWaiting();
      })
      .catch((error) => {
        console.error('[SW] Failed to cache core assets:', error);
      })
  );
});

self.addEventListener('activate', (event) => {
  console.log('[SW] Activating service worker - cleaning up old caches');

  event.waitUntil(
    caches.keys()
      .then((cacheNames) => {
        console.log('[SW] Current caches:', cacheNames);

        return Promise.all(
          cacheNames
            .filter((cacheName) => {
              const shouldDelete = cacheName !== CACHE_NAME;
              if (shouldDelete) {
                console.log(`[SW] Deleting old cache: ${cacheName}`);
              }
              return shouldDelete;
            })
            .map((cacheName) => caches.delete(cacheName))
        );
      })
      .then(() => {
        console.log('[SW] Cache cleanup completed');
        return self.clients.claim();
      })
      .catch((error) => {
        console.error('[SW] Error during cache cleanup:', error);
      })
  );
});

self.addEventListener('fetch', (event) => {
  const request = event.request;
  const url = request.url;

  if (!url.startsWith(self.location.origin)) {
    console.log(`[SW] Skipping cross-origin request: ${url}`);
    return;
  }

  if (url.includes('/api/')) {
    console.log(`[SW] API request (network-first): ${url}`);

    event.respondWith(
      fetch(request)
        .then((response) => {
          if (response.ok && request.method === 'GET') {
            const responseClone = response.clone();
            caches.open(CACHE_NAME)
              .then((cache) => cache.put(request, responseClone))
              .catch((error) => console.log('[SW] Failed to cache API response:', error));
          }
          return response;
        })
        .catch((error) => {
          console.log(`[SW] Network failed, trying cache for: ${url}`);
          return caches.match(request);
        })
    );
    return;
  }

  console.log(`[SW] Static request (cache-first): ${url}`);

  event.respondWith(
    caches.match(request)
      .then((cachedResponse) => {
        if (cachedResponse) {
          console.log(`[SW] Serving from cache: ${url}`);
          return cachedResponse;
        }

        console.log(`[SW] Fetching from network: ${url}`);
        return fetch(request)
          .then((response) => {
            if (response.ok) {
              const responseClone = response.clone();
              caches.open(CACHE_NAME)
                .then((cache) => cache.put(request, responseClone))
                .catch((error) => console.log('[SW] Failed to cache response:', error));
            }
            return response;
          });
      })
      .catch((error) => {
        console.log(`[SW] Network and cache failed for: ${url}`);

        if (request.destination === 'document') {
          console.log('[SW] Serving offline fallback page');
          return caches.match('/index.html');
        }

        if (request.destination === 'image') {
          return new Response(
            '<svg width="1" height="1" xmlns="http://www.w3.org/2000/svg"><rect width="1" height="1" fill="transparent"/></svg>',
            {
              headers: { 'Content-Type': 'image/svg+xml' },
              status: 200
            }
          );
        }

        return new Response(
          JSON.stringify({
            error: 'Offline - Resource not available',
            message: 'This resource is not available offline. Please check your connection.'
          }),
          {
            headers: { 'Content-Type': 'application/json' },
            status: 503
          }
        );
      })
  );
});

self.addEventListener('message', (event) => {
  const { type, payload } = event.data;

  switch (type) {
    case 'SKIP_WAITING':
      self.skipWaiting();
      break;

    case 'CACHE_CLEAR':
      caches.keys().then((cacheNames) => {
        return Promise.all(
          cacheNames.map((cacheName) => caches.delete(cacheName))
        );
      }).then(() => {
        event.ports[0].postMessage({ success: true });
      });
      break;

    case 'CACHE_UPDATE':
      if (payload && payload.urls) {
        caches.open(CACHE_NAME)
          .then((cache) => cache.addAll(payload.urls))
          .then(() => {
            event.ports[0].postMessage({ success: true });
          })
          .catch((error) => {
            event.ports[0].postMessage({ success: false, error: error.message });
          });
      }
      break;

    default:
      console.log('[SW] Unknown message type:', type);
  }
});

self.addEventListener('sync', (event) => {
  if (event.tag === 'background-sync') {
    console.log('[SW] Background sync triggered');
    event.waitUntil(
      Promise.resolve()
    );
  }
});

/*
self.addEventListener('push', (event) => {
  const options = {
    body: event.data.text(),
    icon: '/linux-icon.svg',
    badge: '/linux-icon.svg',
    vibrate: [100, 50, 100],
    data: {
      dateOfArrival: Date.now(),
      primaryKey: 1
    }
  };

  event.waitUntil(
    self.registration.showNotification('Rust Blog CMS', options)
  );
});
*/

self.addEventListener('error', (event) => {
  console.error('[SW] Service Worker error:', event.error);
});

self.addEventListener('unhandledrejection', (event) => {
  console.error('[SW] Unhandled promise rejection:', event.reason);
});

console.log('[SW] Service Worker loaded successfully');
