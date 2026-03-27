# Deploying the Docs Site

The Starlight site lives in `docs-site/` and is deployed as static HTML to `refac.javedab.com` via Caddy.

## Build

From the repo root:

```bash
cd docs-site
bun run build
```

Output: `docs-site/dist/`

## Deploy to Caddy server

Sync the built output to the server's web root. Replace `user` and `your-server` with your actual SSH user and host:

```bash
rsync -avz --delete docs-site/dist/ user@your-server:/var/www/refac/
```

`--delete` removes files on the server that no longer exist locally, keeping the remote in sync with the build.

## Caddyfile

Add this block to your Caddyfile (adjust the web root path to match your server layout):

```
refac.javedab.com {
    root * /var/www/refac
    file_server
    encode zstd gzip
}
```

Caddy handles HTTPS automatically via Let's Encrypt. No certificate management needed.

After editing the Caddyfile, reload Caddy:

```bash
sudo systemctl reload caddy
# or, if running directly:
caddy reload --config /etc/caddy/Caddyfile
```

## DNS

Point an A record (or CNAME to your main domain) for `refac.javedab.com` at your server's IP before the first deploy.

## One-liner build + deploy

```bash
cd docs-site && bun run build && rsync -avz --delete dist/ user@your-server:/var/www/refac/
```
