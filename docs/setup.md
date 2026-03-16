# Buffet — Developer Setup Guide

This guide walks you through setting up the Buffet trading platform for local development, from cloning the repository to running the full stack.

---

## Table of Contents

1. [Prerequisites](#1-prerequisites)
2. [Clone and First Build](#2-clone-and-first-build)
3. [Starting TimescaleDB with Docker](#3-starting-timescaledb-with-docker)
4. [Configuring Environment Variables](#4-configuring-environment-variables)
5. [Database Migrations](#5-database-migrations)
6. [Running the Backend Server](#6-running-the-backend-server)
7. [Running Tests](#7-running-tests)
8. [Building the Frontend](#8-building-the-frontend)

---

## 1. Prerequisites

Ensure the following tools are installed before proceeding.

### Required

| Tool | Version | Install |
|------|---------|---------|
| Rust (stable) | 1.78 or later | https://rustup.rs |
| Docker | 24.x or later | https://docs.docker.com/get-docker/ |
| Docker Compose | v2.x or later | Bundled with Docker Desktop |
| Node.js | 20 LTS or later | https://nodejs.org |
| npm | 10.x or later | Bundled with Node.js |

### Optional but Recommended

| Tool | Purpose |
|------|---------|
| `sqlx-cli` | Run migrations manually, generate query metadata |
| `cargo-watch` | Auto-rebuild on file changes during development |
| `just` | Task runner (if a `justfile` is present) |

Install `sqlx-cli` with SQLite and Postgres support:

```sh
cargo install sqlx-cli --no-default-features --features native-tls,sqlite,postgres
```

Install `cargo-watch`:

```sh
cargo install cargo-watch
```

---

## 2. Clone and First Build

```sh
git clone https://github.com/your-org/buffet.git
cd buffet
```

### Build the backend

```sh
cd buffet-backend
cargo build
```

The first build will take a few minutes as it downloads and compiles all dependencies (including Polars, SQLx, and Axum).

> **Note:** The project uses `sqlx` with compile-time query checking. If you see errors like `error: no DATABASE_URL set`, complete [Section 4](#4-configuring-environment-variables) first, then rebuild.

---

## 3. Starting TimescaleDB with Docker

Buffet uses [TimescaleDB](https://www.timescale.com/) (a PostgreSQL extension) for storing time-series market data.

### Start the container

From the repository root:

```sh
docker compose up -d timescaledb
```

If no `docker-compose.yml` exists yet, you can start the container manually:

```sh
docker run -d \
  --name buffet-timescaledb \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=buffet_timeseries \
  -p 5432:5432 \
  timescale/timescaledb:latest-pg16
```

### Verify the container is running

```sh
docker ps | grep timescaledb
```

### Connect to the database (optional sanity check)

```sh
docker exec -it buffet-timescaledb psql -U postgres -d buffet_timeseries
```

---

## 4. Configuring Environment Variables

The backend reads configuration from environment variables. A template is provided at the repository root.

### Copy the example file

```sh
# From the repository root
cp .env.example buffet-backend/.env
```

### Edit `buffet-backend/.env`

Open `buffet-backend/.env` in your editor and adjust values as needed:

```env
# Required: SQLite database path (relative to the buffet-backend directory)
DATABASE_URL=sqlite:./db/buffet.db

# Required: TimescaleDB connection string
TSDB_URL=postgres://postgres:postgres@localhost:5432/buffet_timeseries

# Server bind address
SERVER_HOST=127.0.0.1
SERVER_PORT=3000

# Actor system tuning
ACTOR_MAILBOX_SIZE=1000
ACTOR_TIMEOUT_MS=5000

# Logging level: trace | debug | info | warn | error
RUST_LOG=info
```

### Create the SQLite database directory

```sh
mkdir -p buffet-backend/db
```

> The `DATABASE_URL` path is resolved relative to where you run `cargo run`, which is typically the `buffet-backend` directory.

---

## 5. Database Migrations

SQLite migrations are applied **automatically on startup** via `sqlx::migrate!()` embedded in the backend binary. You do not need to run them manually under normal circumstances.

### Manual migration (optional)

If you need to inspect or apply migrations manually using `sqlx-cli`:

```sh
cd buffet-backend

# Apply all pending migrations
sqlx migrate run --database-url sqlite:./db/buffet.db

# Check migration status
sqlx migrate info --database-url sqlite:./db/buffet.db
```

### Migration files

All migration files live in `buffet-backend/migrations/` and are applied in filename order:

```
migrations/
  20260202_create_strategies.sql
  20260207_create_signals.sql
  20260207145500_create_orders.sql
  20260220100000_create_positions.sql
  20260220110000_create_backtests.sql
  20260316000001_add_strategy_status.sql
  20260316000002_add_strategy_symbols.sql
```

> **TimescaleDB tables** (hypertables for OHLCV data) are created automatically by the backend on first connection via `db::setup_tsdb_tables()`.

---

## 6. Running the Backend Server

```sh
cd buffet-backend
cargo run
```

The server starts at `http://127.0.0.1:3000` by default (configurable via `SERVER_HOST` and `SERVER_PORT`).

### Development mode (auto-rebuild on changes)

```sh
cargo watch -x run
```

### Verify the server is healthy

```sh
curl http://127.0.0.1:3000/health
# Expected: {"status":"ok"}
```

### Key API endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Health check |
| `GET` | `/strategies` | List all strategies |
| `POST` | `/strategies` | Create a new strategy |
| `GET` | `/strategies/:id` | Get a strategy by ID |
| `PUT` | `/strategies/:id` | Update a strategy |
| `DELETE` | `/strategies/:id` | Delete a strategy |
| `POST` | `/strategies/:id/activate` | Activate a strategy (loads into executor) |
| `POST` | `/strategies/:id/deactivate` | Deactivate a strategy (removes from executor) |
| `GET` | `/orders` | List orders |
| `GET` | `/positions` | List positions |
| `POST` | `/backtests` | Create and run a backtest |

---

## 7. Running Tests

### Unit and integration tests

```sh
cd buffet-backend
cargo test
```

### Run a specific test or module

```sh
# Run all tests in a specific module
cargo test models::strategy

# Run a single test by name
cargo test test_create_and_find_strategy

# Run tests with output visible (do not capture stdout)
cargo test -- --nocapture
```

### Enable test logging

Set `TEST_LOG=1` (or any value) before running tests, and set `RUST_LOG` to the desired level:

```sh
TEST_LOG=1 RUST_LOG=debug cargo test -- --nocapture
```

### Integration tests

Integration tests live in `buffet-backend/tests/`. They spin up an in-memory SQLite database and a real Axum router — no external services required.

```sh
cargo test --test '*'
```

### Check compilation without running

```sh
cargo check
```

### Run linting

```sh
cargo clippy -- -D warnings
```

---

## 8. Building the Frontend

The frontend lives in `buffet-frontend/` and is a standard Node.js/npm project.

### Install dependencies

```sh
cd buffet-frontend
npm install
```

### Start the development server

```sh
npm run dev
```

The frontend dev server typically starts at `http://localhost:5173` (Vite default) or `http://localhost:3001`, depending on the project configuration.

### Build for production

```sh
npm run build
```

Output is placed in `buffet-frontend/dist/`.

### Lint and type-check

```sh
npm run lint
npm run type-check   # if configured
```

---

## Troubleshooting

### `DATABASE_URL` not set during `cargo build`

SQLx performs compile-time query checking and requires a live database or a `sqlx-data.json` offline cache.

**Option A — Use the offline cache** (no running DB needed):

```sh
cargo sqlx prepare
cargo build
```

**Option B — Point to a live database**:

```sh
export DATABASE_URL=sqlite:./db/buffet.db
cargo build
```

---

### TimescaleDB connection refused

Make sure the Docker container is running:

```sh
docker ps | grep timescaledb
docker start buffet-timescaledb   # if stopped
```

---

### Port 3000 already in use

Change the port in `.env`:

```env
SERVER_PORT=3001
```

---

### Rust toolchain is too old

```sh
rustup update stable
```

---

## Project Structure (Reference)

```
buffet/
├── .env.example               # Environment variable template
├── docs/
│   └── setup.md               # This file
├── buffet-backend/
│   ├── src/
│   │   ├── actors/            # Kameo actor implementations
│   │   ├── handlers/          # Axum HTTP handlers
│   │   ├── models/            # SQLx database models
│   │   ├── routes/            # Router configuration
│   │   ├── broker/            # Broker abstraction (paper trading)
│   │   ├── tsdb/              # TimescaleDB helpers
│   │   ├── config.rs          # Configuration loading
│   │   ├── db.rs              # Database setup
│   │   ├── error.rs           # Error types
│   │   ├── state.rs           # Shared application state
│   │   └── main.rs            # Entry point
│   ├── migrations/            # SQLite migration files
│   ├── tests/                 # Integration tests
│   └── Cargo.toml
└── buffet-frontend/
    ├── src/                   # Frontend source
    └── package.json
```
