var staticCacheName = 'stream-watcher'

self.addEventListener('install', function (e) {
  e.waitUntil(
    caches.open(staticCacheName).then(function (cache) {
      return cache.addAll(['/'])
    })
  )
})

self.addEventListener('fetch', function (event) {
  if (event.request.url.startsWith(self.origin)) {
    fetch(event.request.url).then(
      (resp) => {
        return resp
      },
      () => {
        event.respondWith(
          caches.match(event.request).then(function (response) {
            return response
          })
        )
      }
    )
  } else {
    event.respondWith(
      caches.match(event.request).then(function (response) {
        return response || fetch(event.request)
      })
    )
  }
})
