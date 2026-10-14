#!/bin/bash

# An explicit empty sitemap is the only honest comparison baseline before a
# search engine has received the site. Never use the current release as both
# sides of a bootstrap comparison.
ci_indexnow_create_empty_release() {
  local release_root="$1"
  local sitemap_path="$2"

  mkdir -p "$release_root/$(dirname "$sitemap_path")"
  printf '%s\n' \
    '<?xml version="1.0" encoding="UTF-8"?>' \
    '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"></urlset>' \
    > "$release_root/$sitemap_path"
}
