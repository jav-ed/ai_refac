# Fnox — Environments and Overrides

## Profiles (dev / staging / prod)

Profiles let you manage different secrets for different environments in a single `fnox.toml`.

```toml
# Default profile (development)
[secrets]
DATABASE_URL = { provider = "age", value = "encrypted-dev-db..." }
API_URL = { default = "http://localhost:3000" }

# Staging profile — overrides DATABASE_URL, inherits everything else
[profiles.staging.secrets]
DATABASE_URL = { provider = "age", value = "encrypted-staging-db..." }
API_URL = { default = "https://staging.example.com" }

# Production profile
[profiles.production.secrets]
DATABASE_URL = { provider = "age", value = "encrypted-prod-db..." }
API_URL = { default = "https://api.example.com" }
```

Profiles inherit from the default — only override what differs.

### Using profiles

```bash
# via flag
fnox exec --profile staging -- ./deploy.sh
fnox get DATABASE_URL --profile production

# via env var (for the whole session)
export FNOX_PROFILE=staging
fnox exec -- node server.js
```

### List available profiles

```bash
fnox profiles
```

---

## Hierarchical config (monorepos)

Fnox walks up the directory tree and merges all `fnox.toml` files it finds. Child configs override parent configs.

```
project/
├── fnox.toml              ← shared: providers, JWT_SECRET, LOG_LEVEL
└── services/
    ├── api/
    │   └── fnox.toml      ← api-specific: DATABASE_URL, API_PORT
    └── worker/
        └── fnox.toml      ← worker-specific: QUEUE_URL, CONCURRENCY
```

From `services/api/`, both the root and api-level secrets are available. The api-level value wins on conflicts.

**Stop tree-walking at a boundary:**
```toml
# In a subdirectory fnox.toml — prevents searching further up
root = true
```

---

## Local overrides (fnox.local.toml)

For personal or machine-specific overrides that should never be committed:

```bash
# Add to .gitignore
echo "fnox.local.toml" >> .gitignore
```

```toml
# fnox.local.toml — gitignored, highest priority at this level
[secrets]
DATABASE_URL = { default = "postgresql://localhost/mylocal" }
```

**Common uses:**
- Point `DATABASE_URL` at a local dev database
- Override an API key with a personal test key
- Machine-specific paths or tokens

Provide a committed `fnox.local.toml.example` as a template for teammates.

---

## Global config

For personal tokens that apply across all projects (GitHub, npm, etc.):

```bash
fnox init --global
fnox set GITHUB_TOKEN "ghp_..." --global
```

Stored at `~/.config/fnox/config.toml`. Always loaded, even when `root = true` stops parent-dir recursion.
