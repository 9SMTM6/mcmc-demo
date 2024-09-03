const CACHE_NAME = 'mmcmc-demo-v1';
const STATIC_FILES_REGEX = [
    /^\/$/,
    /^\/(index\.html)$/,
    /^\/mcmc-demo-[a-f0-9]{16}_bg\.wasm$/,
    /^\/mcmc-demo-[a-f0-9]{16}\.js$/,
    /^\/snippets\/wasm-bindgen-futures-[a-f0-9]{16}\/src\/task\/worker.js$/,
    /^\/favicon-[a-f0-9]{16}\.svg$/,
    /^\/favicon.svg$/,
    /^\/manifest.json$/,
];

// Fetch event: Serve cached files, and cache new ones with hashes
self.addEventListener('fetch', (event) => {
    const url = new URL(event.request.url);

    if (url.origin === location.origin) {
        const matchingRegex = STATIC_FILES_REGEX.find(regex => regex.test(url.pathname));

        if (matchingRegex) {
            event.respondWith(
                caches.match(event.request).then((cachedResponse) => {
                    // If cached, return it
                    if (cachedResponse) {
                        return cachedResponse;
                    }
                    // Else, fetch from network and cache the response
                    return caches.open(CACHE_NAME).then((cache) => {
                        return fetch(event.request).then((response) => {
                            cache.put(event.request, response.clone());
                            return response;
                        });
                    });
                })
            );
        } else {
            // Handle other requests normally (or apply different caching strategy)
            event.respondWith(fetch(event.request));
        }
    } else {
        // Handle third-party requests normally
        event.respondWith(fetch(event.request));
    }
});

// Activate event: Clean up old caches
self.addEventListener('activate', (event) => {
    self.clients.claim();
    event.waitUntil(
        caches.keys().then((cacheNames) => {
            return Promise.all(
                cacheNames.map((cacheName) => {
                    if (cacheName !== CACHE_NAME) {
                        return caches.delete(cacheName);
                    }
                })
            );
        })
    );
});
