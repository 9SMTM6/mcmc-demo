// Register Service Worker. this will cache the wasm / js scripts for offline use (for PWA functionality).
// It should always get the most up to date files (if connected to the server), but you can Force refresh (Ctrl + F5) to enforce that. 
if ('serviceWorker' in navigator) {
    navigator.serviceWorker
        .register('serviceworker.js')
        // update service worker
        .then((registration) => registration.update())
        // don't log errors in console when there's no network connection.
        .catch(() => {});
}
