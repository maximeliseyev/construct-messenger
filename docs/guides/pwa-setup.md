# PWA Setup and Offline Capabilities

This guide covers the core components that make the application a Progressive Web App (PWA), focusing on installation, offline functionality, and background processing.

## 1. Service Workers

The Service Worker is a background script that acts as a network proxy, enabling offline experiences, handling push notifications, and performing background tasks.

### Registration
The Service Worker must be registered when the application starts.

```typescript
// main.ts
if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/sw.js', {
      scope: '/',
    }).then(registration => {
      console.log('ServiceWorker registration successful with scope: ', registration.scope);
    }).catch(error => {
      console.log('ServiceWorker registration failed: ', error);
    });
  });
}
```

### Caching Strategies
We use a **cache-first** strategy for static assets and a **network-first** strategy for API calls.

```typescript
// sw.js

// On install, cache static assets (app shell)
self.addEventListener('install', event => {
  event.waitUntil(
    caches.open(CACHE_NAME).then(cache => {
      return cache.addAll([
        '/',
        '/index.html',
        '/styles.css',
        '/app.js',
        '/manifest.json',
        '/crypto.wasm', // Cache the WASM module
      ]);
    })
  );
});

// On fetch, apply caching strategy
self.addEventListener('fetch', event => {
  if (event.request.url.includes('/api/')) {
    // Network-first for API requests
    event.respondWith(
      fetch(event.request).catch(() => {
        return caches.match(event.request); // Fallback to cache if offline
      })
    );
  } else {
    // Cache-first for static assets
    event.respondWith(
      caches.match(event.request).then(response => {
        return response || fetch(event.request);
      })
    );
  }
});
```

## 2. Web App Manifest

The `manifest.json` file provides the browser with metadata about the PWA, making it installable.

```json
{
  "short_name": "Construct",
  "name": "Construct Messenger",
  "icons": [
    {
      "src": "/icons/icon-192x192.png",
      "type": "image/png",
      "sizes": "192x192"
    },
    {
      "src": "/icons/icon-512x512.png",
      "type": "image/png",
      "sizes": "512x512"
    }
  ],
  "start_url": ".",
  "display": "standalone",
  "theme_color": "#000000",
  "background_color": "#ffffff"
}
```

## 3. Offline Operation

True offline capability requires more than just caching the app shell.

-   **Message Queuing**: When the user sends a message while offline, it should be added to an "outbox" queue in **IndexedDB**.
-   **Background Sync**: The **Background Sync API** is used to detect when connectivity is restored. The Service Worker listens for the `sync` event and processes the outbox, sending any pending messages.

```typescript
// In the main app, when sending a message
async function sendMessage(message: Message) {
  if (navigator.onLine) {
    await api.sendMessage(message);
  } else {
    // Add to outbox in IndexedDB
    await db.addToOutbox(message);
    // Register a sync event
    const registration = await navigator.serviceWorker.ready;
    await registration.sync.register('send-outbox');
  }
}

// In sw.js
self.addEventListener('sync', event => {
  if (event.tag === 'send-outbox') {
    event.waitUntil(processOutbox());
  }
});

async function processOutbox() {
  const outboxMessages = await db.getOutbox();
  for (const message of outboxMessages) {
    try {
      await api.sendMessage(message);
      await db.removeFromOutbox(message.id);
    } catch (error) {
      console.error('Failed to send message from outbox', error);
      // Decide whether to retry or fail
      break; // Stop on first failure to maintain order
    }
  }
}
```

This combination of Service Workers, caching, and background sync provides a robust and resilient user experience, even with intermittent network connectivity.
