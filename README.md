# ksecret

**ksecret** is a CLI tool that makes managing Kubernetes secrets less painful. It uses Google Cloud Secret Manager as the source of truth and syncs your secrets to your Kubernetes clusters with a single command.

Stop manually base64-encoding strings or committing `.env` files to git. Use `ksecret` to keep your dev, staging, and prod environments consistent and secure.

## Why?

Managing secrets across multiple clusters is often a mess of ad-hoc scripts or manual copy-pasting. `ksecret` gives you:
- **One standardized format:** Secrets in GCP are named `k8s-{env}-{name}` (e.g. `k8s-prod-db-pass`).
- **One command sync:** `ksecret sync prod` pulls all production secrets and applies them to your cluster.
- **Reliability:** Uses a "delete-then-create" strategy to ensure your cluster state exactly matches GCP.

## Installation

```bash
# From source
cargo install --path .
```

## Quick Start

### 1. Initialize
Tell `ksecret` which GCP project to use.
```bash
ksecret init --project my-gcp-project
```

### 2. Create a Secret
Push a secret to Google Cloud Secret Manager.
```bash
# Interactive (avoids history)
ksecret set --env dev db-password --value "super-secret-123"

# Or from stdin
echo "my-api-key" | ksecret set --env dev api-key --stdin
```

### 3. Sync to Kubernetes
Switch to your target cluster context and sync.
```bash
# Sync 'dev' secrets to the 'default' namespace
ksecret sync dev

# Sync to a specific namespace
ksecret sync staging --namespace backend-services
```

## Commands

| Command | Description |
|---------|-------------|
| `init` | Set up your local config (project ID, etc). |
| `set` | Create or update a secret in GCP. |
| `get` | Fetch a secret value from GCP. |
| `list` | Show all secrets for a specific environment. |
| `delete` | Remove a secret from GCP. |
| `sync` | Download secrets for an env and apply them to K8s. |

## Tips

- **Dry Run:** Use `--dry-run` with `sync` to see what would happen without making changes.
- **Contexts:** You can specify a different kube context with `-c` / `--context` if you don't want to switch your active context.
- **Local Config:** You can override the config file location with `KSECRET_CONFIG_FILE` if needed.

## License

MIT
