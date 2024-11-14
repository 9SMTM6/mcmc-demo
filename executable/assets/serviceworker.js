const CACHE_NAME = 'mmcmc-demo-v1';
const HASHED_FILES = [
    /^(\/fat|\/slim)?\/mcmc-demo-[a-f0-9]{16}_bg\.wasm$/,
    /^(\/fat|\/slim)?\/mcmc-demo-[a-f0-9]{16}\.js$/,
    /^(\/fat|\/slim)?\/snippets\/wasm-bindgen-futures-[a-f0-9]{16}\/src\/task\/worker\.js$/,
    /^(\/fat|\/slim)?\/snippets\/wasm-bindgen-rayon-[a-f0-9]{16}\/src\/workerHelpers\.no-bundler\.js$/,
    /^(\/fat|\/slim)?\/snippets\/wasm-bindgen-rayon-[a-f0-9]{16}\/src\/workerHelpers\.js$/,
    /^(\/fat|\/slim)?\/snippets\/wasm-bindgen-rayon-[a-f0-9]{16}\/src\/workerHelpers\.worker\.js$/,
];
const UNHASHED_FILES = [
    "/",
    "/index.html",
    "/favicon.svg",
    "/manifest.json",
]

// Fetch event: Serve cached files, and update old ones / cache new ones
self.addEventListener('fetch', (event) => {
    const url = new URL(event.request.url);

    if (url.origin === location.origin) {
        const matchingFilename = UNHASHED_FILES.find(filename => url.pathname.endsWith(filename) );
        const matchingRegexOrFilename = matchingFilename || HASHED_FILES.find(regex => regex.test(url.pathname));
        
        if (matchingRegexOrFilename) {
            event.respondWith((async () => {
                const cachedResponse = await caches.match(event.request);
                if (cachedResponse) {
                    // if its a hashed file, returning it is (pretty much, exclusing hash conflicts which I'll ignore) save to just serve the cached file.
                    if (!matchingFilename) {
                        return cachedResponse;
                    } 
                    // If the element is not hashed for cache invalidation, try to refresh the cache.
                    let networkResponse = await fetch(event.request).catch(() => cachedResponse);
                    if (!networkResponse.ok) {
                        return cachedResponse;
                    }
                    // we leave the cache update dangling, no need to wait for that.
                    caches.open(CACHE_NAME).then((cache) => {
                        // I don't clone here as, from my undersanding, this should reliably be executed after the return of the function.
                        // So after the original request was already cloned successfully.
                        cache.put(event.request, networkResponse);
                    });
                    return networkResponse.clone();
                }
                const networkResponse = await fetch(event.request);
                // we leave the cache update dangling, no need to wait for that.
                if (networkResponse.ok) {
                    caches.open(CACHE_NAME).then((cache) => {
                        cache.put(event.request, networkResponse);
                    });    
                } 
                return networkResponse.clone();
            })());
        }
        // If we don't invoke `event.respondWith` then the original fetch goes ahead unperturbed. 
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
