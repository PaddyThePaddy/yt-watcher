var staticCacheName = 'stream-watcher'

self.addEventListener('install', function (e) {
  e.waitUntil(
    caches.open(staticCacheName).then(function (cache) {
      console.log('add cache')
      return cache.addAll(['/', '/assets/index.css', '/assets/index.js'])
    })
  )
})

self.addEventListener('fetch', function (event) {
  event.respondWith(
    caches.match(event.request).then(function (response) {
      return response || fetch(event.request)
    })
  )
})
