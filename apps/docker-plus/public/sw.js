// Service Worker: Optimized for Query-String Interception
self.addEventListener('install', (event) => { self.skipWaiting(); });
self.addEventListener('activate', (event) => { event.waitUntil(clients.claim()); });

self.addEventListener('fetch', (event) => {
    const url = new URL(event.request.url);

    // Intercept if the URL contains "?api=dashboard"
    if (url.searchParams.has('api') && url.searchParams.get('api') === 'dashboard') {
        event.respondWith(
            new Response(JSON.stringify({
                user: { id: "Syed-Ismaeel", reputation: 24850, balance: "12.5M" },
                feed: [
                    { name: "Lion-Explorer", author: "Syedsmaeel", service: "gitlab", audit: { status: "PASSED" }, timestamp: new Date().toISOString() },
                    { name: "openclaw-engine", author: "Syedsmaeel", service: "github", audit: { status: "PASSED" }, timestamp: new Date().toISOString() }
                ],
                status: "success"
            }), {
                headers: { 'Content-Type': 'application/json' }
            })
        );
    }
});
