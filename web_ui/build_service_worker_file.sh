#!/bin/bash

cat >$TRUNK_STAGING_DIR/service_worker.js << EOL
var cacheName = 'handy-solver';
var filesToCache = [
$(cd $TRUNK_STAGING_DIR && find . -type f | xargs -I@ echo "'@',")
];


self.addEventListener('install', function(e) {
  e.waitUntil(
    caches.open(cacheName).then(function(cache) {
      return cache.addAll(filesToCache);
    })
  );
});

self.addEventListener('fetch', function(e) {
  e.respondWith(
    caches.match(e.request).then(function(response) {
      return response || fetch(e.request);
    })
  );
});
EOL
