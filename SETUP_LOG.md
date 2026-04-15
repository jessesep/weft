# Weft Staging Setup Log - Spark

Date: 2026-04-15
Host: Spark (100.64.4.86, aarch64, Linux 6.14.0-1015-nvidia)
Location: /home/one-admin/staging/weft/

## What Was Done

### 1. Cloned Repository

```
git clone https://github.com/WeaveMindAI/weft.git /home/one-admin/staging/weft/
```

### 2. System Prerequisites - Already Available

- Node.js v24.13.0
- npm 11.6.2
- Docker 28.5.1
- Bash 5.2.21 (meets Bash 4+ requirement)
- Architecture: aarch64

### 3. Installed Missing Dependencies

**Rust (via rustup):**
- rustc 1.97.0-nightly (a5c825cd8 2026-04-14)
- cargo 1.97.0-nightly (eb94155a9 2026-04-09)
- Already had rustup installed with nightly toolchain, `rustup` just updated it

**pnpm:**
- pnpm 10.33.0
- Installed to /home/one-admin/.local/share/pnpm/

**Restate Server + CLI:**
- restate-cli 1.6.2
- restate-server 1.6.2
- Installed via `npm install --global @restatedev/restate-server@latest @restatedev/restate@latest`
- Binaries at /home/one-admin/.nvm/versions/node/v24.13.0/bin/

### 4. Port Conflict Resolution

Ports 8080 and 3000 were already in use on Spark:
- 8080: Python process (existing service)
- 3000: Next.js server (existing service)

Custom ports configured in `.env`:
- RESTATE_PORT=18080 (was 8080)
- RESTATE_ADMIN_PORT=19070 (was 9070)
- RESTATE_RPC_PORT=15122 (was 5122)
- WEFT_API_PORT=13000 (was 3000)

### 5. PostgreSQL via Docker

- Container: `weft-local-postgres` (postgres:16)
- Port: 5433 (no conflict)
- Volume: weft-local-pgdata
- Schema: 13 tables initialized via init-db.sql
- CONNECTION: postgres://postgres:postgres@localhost:5433/weft_local

### 6. Build

```
bash scripts/catalog-link.sh   # 97 Rust + 98 TS nodes linked
cargo build --release           # 353 crates, ~64 seconds
```

Three binaries built:
- target/release/orchestrator (25.6M)
- target/release/weft-api (25.6M)
- target/release/node-runner (20.7M)

### 7. Services Started

All services run in tmux session `weft` (5 windows).

| Service | Port | Status | tmux Window |
|---------|------|--------|-------------|
| Restate Server (ingress) | 18080 | Running | restate |
| Restate Admin | 19070 | Running | restate |
| Restate RPC | 15122 | Running | restate |
| Orchestrator | 9080 | Running | orchestrator |
| Weft API | 13000 | Running | api |
| Node Runner | 9082 | Running (97 node types) | node-runner |
| Dashboard (SvelteKit) | 5174 | Running | dashboard |
| PostgreSQL | 5433 | Running (Docker) | - (container) |

### 8. Dashboard Configuration

Created `dashboard/.env` with:
- DATABASE_URL pointing to local PostgreSQL
- API_URL=http://localhost:13000 (custom port)

## How to Use

### Access the dashboard
Open http://localhost:5174 (or http://100.64.4.86:5174 from Tailscale - note: dashboard listens on 127.0.0.1 only by default)

### Manage services
```bash
tmux attach -t weft              # Attach to session
tmux select-window -t weft:0     # Switch to restate window
tmux select-window -t weft:1     # Switch to orchestrator window
tmux select-window -t weft:2     # Switch to api window
tmux select-window -t weft:3     # Switch to node-runner window
tmux select-window -t weft:4     # Switch to dashboard window
```

### Stop everything
```bash
tmux kill-session -t weft        # Kill all backend services
docker stop weft-local-postgres  # Stop PostgreSQL
```

### Restart from scratch
```bash
cd /home/one-admin/staging/weft
bash cleanup.sh --no-db          # Stop services, clean Restate data
bash cleanup.sh                  # Stop everything including DB
```

## Logs

All service logs are in `/home/one-admin/staging/weft/logs/`:
- restate.log
- orchestrator.log
- api.log
- node-runner.log
- dashboard.log

## Known Issues / Notes

1. **Weft API usage backfill warning** - Non-critical: "Failed to backfill usage aggregation: unexpected null". This is a minor DB schema edge case on fresh installs.

2. **No API keys configured** - LLM nodes, web search, Discord, etc. will show errors at runtime until API keys are added to `.env`. All keys are optional per the README.

3. **Dashboard binds to 127.0.0.1** - Not accessible from other machines on Tailscale by default. To expose, modify the dashboard dev command to use `--host 0.0.0.0`.

4. **Restate config file** - Custom config at `/home/one-admin/staging/weft/restate-config.toml` for non-default ports.

5. **No Kubernetes** - Infrastructure nodes (like PostgresDatabase) require `kind` for local K8s. Not set up since it was not needed for basic functionality.

6. **Code sandbox disabled** - `CODE_SANDBOX_ENABLED=false` in .env (nsjail not installed).

## Restate registration

The orchestrator is registered with Restate. Deployed services:
- InfrastructureManager (rev 1)
- TaskRegistry (rev 1)
- NodeInstanceRegistry (rev 1)
