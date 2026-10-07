# Production Handoff

Give the deploying colleague:

- repository and branch;
- exact build and deployable directory (`dist/client`);
- runtime release path and `main` symlink;
- tracked Caddy domain file;
- health URL and one deep docs URL;
- DNS record to activate only after the server responds correctly.

They perform the first release copy, server-side Caddy validation, reload, and DNS activation. Record the deployed commit and verification result afterward.
