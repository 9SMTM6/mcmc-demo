const CACHE_NAME = 'mmcmc-demo-v1';
const HASHED_FILES = [
    /^\/mcmc-demo-[a-f0-9]{16}_bg\.wasm$/,
    /^\/mcmc-demo-[a-f0-9]{16}\.js$/,
    /^\/snippets\/wasm-bindgen-futures-[a-f0-9]{16}\/src\/task\/worker.js$/,
    // TODO: really I'd like to have the webpage icon not hashed with how stuff is handled.
    // But it doesn't seem trunk offers that option.
    /^\/favicon-[a-f0-9]{16}\.svg$/,
];
const UNHASHED_FILES = [
    "/",
    "/index.html",
    "/favicon.svg",
    "/manifest.json",
]

// Fetch event: Serve cached files, and cache new ones with hashes
self.addEventListener('fetch', (event) => {
    const url = new URL(event.request.url);

    if (url.origin === location.origin) {
        const matchingFilename = UNHASHED_FILES.find(filename => url.pathname === filename);
        const matchingRegexOrFilename = matchingFilename ?? HASHED_FILES.find(regex => regex.test(url.pathname));

        if (matchingRegexOrFilename) {
            event.respondWith(
                caches.match(event.request).then((cachedResponse) => {
                    // If cached, directly return hashed files, however try refreshing unhashed files.
                    if (cachedResponse) {
                        // if its a hashed file, returning it is (pretty much, exclusing hash conflicts which I'll ignore) save to just serve the cached file.
                        if (!matchingFilename) {
                            return cachedResponse;
                        } else {
                            // If the element is not hashed for cache invalidation, try to refresh the cache.
                            return fetch(event.request).then((networkResponse) => {
                                // If the network response is ok, cache the response
                                if (networkResponse.ok) {
                                    caches.open(CACHE_NAME).then((cache) => {
                                        cache.put(event.request, networkResponse.clone());
                                    });
                                    return networkResponse;
                                }
                                // If the network request fails, return the cached response (if available)
                                return cachedResponse || networkResponse;
                            }).catch(() => {
                                // If network fetch fails return cached response
                                return cachedResponse;
                            });
                        }
                    }
                    // Else, fetch from network and cache the response
                    return caches.open(CACHE_NAME).then((cache) => {
                        return fetch(event.request).then((networkResponse) => {
                            cache.put(event.request, networkResponse.clone());
                            return networkResponse;
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
