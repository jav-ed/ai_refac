# Fnox — Credential Leases

Leases vend short-lived credentials from cloud providers (AWS STS, GCP, Vault, Azure). Instead of storing long-lived access keys, fnox creates temporary credentials on demand that expire automatically.

**Skip this unless you are using a cloud provider.** For age-encrypted secrets in git, leases are not relevant.

---

## When to use leases

- You need to call AWS, GCP, Azure, or Vault APIs
- You want short-lived credentials instead of long-lived API keys
- You need to assume an IAM role

---

## Basic setup (AWS STS example)

```toml
# fnox.toml

[providers.age]
type = "age"
recipients = ["ssh-ed25519 AAAA..."]

# Master credentials (long-lived) — stored encrypted
[secrets]
AWS_ACCESS_KEY_ID = { provider = "age", value = "encrypted..." }
AWS_SECRET_ACCESS_KEY = { provider = "age", value = "encrypted..." }

# Lease — uses the master creds to get short-lived role credentials
[leases.aws]
type = "aws-sts"
region = "us-east-1"
role_arn = "arn:aws:iam::123456789012:role/dev-role"
duration = "1h"
```

```bash
# fnox exec: decrypts master creds → calls AssumeRole → injects short-lived creds
fnox exec -- aws s3 ls
```

Leases are cached until within 5 minutes of expiry, then automatically renewed.

---

## Supported backends

| Backend | Type | Max duration |
|---------|------|-------------|
| AWS STS | `aws-sts` | 12h |
| GCP IAM | `gcp-iam` | 1h |
| Azure Token | `azure-token` | ~1h |
| HashiCorp Vault | `vault` | 24h |
| Cloudflare | `cloudflare` | 24h |
| GitHub App | `github-app` | 1h |
| Custom command | `command` | 24h |

---

## Interactive / prompt-based (no stored credentials)

For remote servers where you don't want master credentials on disk at all:

```toml
# fnox.toml — only the lease backend, no secrets needed
[leases.aws]
type = "aws-sts"
region = "us-east-1"
role_arn = "arn:aws:iam::123456789012:role/dev-role"
duration = "1h"
```

```bash
# Create lease interactively (paste credentials from password manager)
fnox lease create aws -i

# All subsequent exec calls use the cached lease
fnox exec -- terraform plan
```

---

## Managing leases

```bash
fnox lease list --active
fnox lease list --expired
fnox lease revoke <lease-id>
fnox lease cleanup      # remove all expired
```
