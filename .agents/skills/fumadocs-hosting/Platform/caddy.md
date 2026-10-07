# Caddy

Serve the current release symlink and prefer precompressed Brotli, Zstandard, then gzip files.

Cache policy:

- hashed `/assets/*`: one year and immutable;
- HTML and `_shell.html`: revalidate;
- Pagefind entry files: revalidate because their names may be stable;
- fonts and icons: short cache unless filenames are content-hashed.

Resolve real files first, then fall back to `_shell.html` for client-side routes. Import the shared security headers. Validate the full Caddy configuration before reload; validation does not authorize deployment or reload.
