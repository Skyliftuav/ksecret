# ksecret

**Kubernetes secrets, simplified.**

`ksecret` is a lightweight CLI that manages your Kubernetes secrets with zero friction. It treats Google Cloud Secret Manager as the source of truth and syncs your secrets to your clusters with a single command.

Stop manually base64-encoding strings or committing `.env` files to git. Use `ksecret` to keep your dev, staging, and prod environments consistent and secure.

## ✨ Features

-   **One Source of Truth**: Manage secrets in GCP, sync to any cluster.
-   **Smart Caching**: Local caching (5-minute TTL) keeps CLI tools fast without hitting API limits.
-   **Standardized Format**: Secrets are automatically named `k8s-{env}-{name}` (e.g. `k8s-prod-db-pass`).
-   **Secure**: Uses a "delete-then-create" strategy to ensure your cluster state exactly matches GCP.
-   **Interactive**: Set secrets without leaving a trace in your shell history.

## 🚀 Getting Started

### Installation

```bash
cargo install --path .
```

### Initialization

Tell `ksecret` which GCP project to use:

```bash
ksecret init --project my-gcp-project
```

### 1. Create a Secret

Push a secret to Google Cloud Secret Manager.

```bash
# Interactive (avoids history)
ksecret set --env dev db-password

# Or from stdin
echo "super-secret-123" | ksecret set --env dev api-key --stdin
```

This updates the local cache instantly, so subsequent reads are fast!

### 2. Read a Secret

Fetch a secret value.

```bash
# Read from cache (fast!)
ksecret get --env dev db-password

# Force refresh from GCP
ksecret get --env dev db-password --no-cache
```

### 3. Sync to Kubernetes

Switch to your target cluster context and sync all secrets for an environment.

```bash
# Sync 'dev' secrets to the 'default' namespace
ksecret sync dev

# Sync to a specific namespace
ksecret sync staging --namespace backend-services
```

## ⚡ Caching

To keep things snappy, `ksecret` caches values locally in `~/.config/ksecret/cache.json` for **5 minutes**.

*   **Reads (`get`)**: Check cache first. If missing or expired, fetch from GCP and update cache.
*   **Writes (`set`)**: Update GCP *and* the local cache immediately.
*   **Deletes (`delete`)**: Remove from GCP *and* the local cache immediately.
*   **Bypass**: Use `--no-cache` to force a direct fetch from GCP.

## 🎮 Commands

| Command | Description |
| :--- | :--- |
| `init` | Set up your local config (project ID, etc). |
| `set` | Create or update a secret in GCP + Cache. |
| `get` | Fetch a secret value (Cache first). |
| `list` | Show all secrets for a specific environment. |
| `delete` | Remove a secret from GCP + Cache. |
| `sync` | Download secrets for an env and apply them to K8s. |

## 💡 Tips

-   **Dry Run:** Use `--dry-run` with `sync` to see what would happen without making changes.
-   **Contexts:** You can specify a different kube context with `-c` / `--context` if you don't want to switch your active context.
-   **Local Config:** You can override the config file location with `KSECRET_CONFIG_FILE` if needed.

---

*Part of the Adriftdev Toolchain.*
