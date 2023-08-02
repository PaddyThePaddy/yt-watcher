var staticCacheName = 'stream-watcher'

function add_cache(cache) {
  console.log('add cache')
  return cache.addAll(['/', '/assets/index.css', '/assets/index.js'])
}

self.addEventListener('install', function (e) {
  e.waitUntil(caches.open(staticCacheName).then(add_cache))
})

self.addEventListener('fetch', function (event) {
  event.respondWith(
    caches.match(event.request).then(function (response) {
      return response || fetch(event.request)
    })
  )
})

self.addEventListener('sync', (event) => {
  console.log('sync')
  if (event.tag == 'refresh_cache') {
    caches.open(staticCacheName).then(add_cache)
  }
})
