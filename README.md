# EGP Closed-Loop Cryptocurrency Exchange

A production-grade cryptocurrency exchange platform priced in **Egyptian Pound (EGP)**, with a closed-loop fiat flow (no third-party payment gateway), live Binance price feeds, an in-memory matching engine, and a fully isolated admin panel for manual deposit/withdrawal processing.

> **Disclaimer:** This is a reference implementation. Running a real cryptocurrency exchange requires regulatory licensing, AML/KYC compliance, custody solutions, and security audits. Use at your own risk.

---

## Architecture Overview

```
┌────────────────────────────────────────────────────────────────────────────┐
│                            Host / Reverse Proxy                            │
└────────────────────────────────────────────────────────────────────────────┘
        │                              │                              │
        ▼                              ▼                              ▼
┌─────────────────┐         ┌──────────────────┐         ┌──────────────────┐
│ User Frontend   │         │ Rust Backend     │         │ Admin Frontend   │
│ SvelteKit :3000 │ ◀────▶  │ Axum :8080       │ ◀────▶  │ SvelteKit :3001  │
└─────────────────┘         └──────────────────┘         └──────────────────┘
                                     │  │  │
                ┌────────────────────┘  │  └──────────────────────┐
                ▼                       ▼                         ▼
       ┌─────────────────┐   ┌──────────────────┐      ┌──────────────────┐
       │  PostgreSQL 16  │   │  Redis 7         │      │  Binance WS      │
       │  Users/Wallets/ │   │  Manual tx queue │      │  Price feed      │
       │  Orders/Trades  │   │  Pub/Sub + cache │      │  (circuit brkr)  │
       └─────────────────┘   └──────────────────┘      └──────────────────┘
```

### Key Design Decisions

| Concern | Choice | Why |
|---|---|---|
| Backend language | Rust (Axum + Tokio + SQLx) | Type safety, zero-cost async, deterministic latency for the matching engine |
| Matching engine | In-memory `BTreeMap` per pair, `parking_lot::RwLock` | O(log n) price-level lookup, O(1) FIFO at each price level |
| Trade persistence | Async, per-trade row inserted via SQLx | Engine stays non-blocking; DB is the source of truth for ledger |
| Price feed | Binance public WebSocket only | No trading account / API key required, real-time bookTicker stream |
| Circuit breaker | Trips after `MAX_FAILURES` parse errors or `TIMEOUT_SECS` of silence | Halts trading to protect users when feed is unhealthy |
| EGP pricing | Derived: `Binance USDT price × EGP/USD rate` | No direct crypto/EGP pair on Binance; rate configurable in `settings` table |
| Fiat flow | Manual bank transfer + admin approval | Closed-loop, no payment gateway integration required |
| Crypto withdrawals | Manual broadcast by admin after review | Provides compliance oversight; locked funds until release |
| Frontend isolation | Two completely separate SvelteKit apps | Admin never shares state/code with user-facing app; can be deployed to different origins |
| Deployment | Distroless Rust image + Alpine Node images | Smallest attack surface; no shell in production backend |

---

## Repository Layout

```
.
├── backend/                          # Rust backend (Axum + SQLx + Tokio)
│   ├── Cargo.toml
│   ├── Dockerfile                    # Multi-stage, distroless final image
│   ├── migrations/
│   │   └── 20240101000001_init.sql   # Full schema (users, wallets, orders, trades, manual_tx, ledger, audit)
│   └── src/
│       ├── main.rs                   # Entry point, wires all subsystems
│       ├── config.rs                 # Env-var based configuration
│       ├── error.rs                  # Unified AppError + IntoResponse
│       ├── models/                   # Domain models + DTOs
│       ├── db/                       # SQLx queries per entity
│       ├── auth/                     # JWT config + Axum extractors (AuthUser, AdminUser)
│       ├── matching_engine/          # BTreeMap-based order book + engine
│       ├── binance/                  # WS client + circuit breaker
│       ├── redis/                    # Queue + pub/sub for manual tx
│       ├── services/                 # Trade pair metadata + fee schedule
│       └── api/                      # HTTP routes
│           ├── auth.rs               # /api/auth/*
│           ├── user.rs               # /api/user/me
│           ├── trading.rs            # /api/user/orders, /api/market/*
│           ├── wallet.rs             # /api/user/wallets, deposits, withdrawals
│           ├── admin.rs              # /api/admin/* (admin-only)
│           └── ws.rs                 # /api/market/ws (multiplexed WS)
├── frontend-user/                    # SvelteKit user frontend (port 3000)
│   ├── package.json
│   ├── Dockerfile
│   ├── svelte.config.js
│   ├── tailwind.config.js
│   ├── src/
│   │   ├── app.html, app.css
│   │   ├── lib/
│   │   │   ├── api.ts                # Typed API client
│   │   │   ├── types.ts              # Shared types matching backend models
│   │   │   ├── stores.ts             # Svelte stores + WS connection
│   │   │   ├── format.ts             # Decimal-safe formatters
│   │   │   └── components/
│   │   │       ├── CircuitBanner.svelte
│   │   │       ├── OrderBook.svelte
│   │   │       ├── RecentTrades.svelte
│   │   │       ├── PriceChart.svelte
│   │   │       └── OrderForm.svelte
│   │   └── routes/
│   │       ├── +layout.svelte        # Nav, header, footer
│   │       ├── +page.svelte          # Dashboard
│   │       ├── login/+page.svelte
│   │       ├── register/+page.svelte
│   │       ├── trade/[pair]/+page.svelte   # Trade view
│   │       ├── wallet/+page.svelte         # Wallet + deposit/withdraw
│   │       └── history/+page.svelte        # Orders + trades history
├── frontend-admin/                   # SvelteKit admin frontend (port 3001)
│   ├── (same structure as frontend-user)
│   └── src/routes/
│       ├── +layout.svelte            # Sidebar admin nav
│       ├── +page.svelte              # Overview
│       ├── login/+page.svelte        # Admin-only login
│       ├── deposits/+page.svelte     # Pending EGP deposits queue
│       ├── withdrawals/+page.svelte  # Pending crypto withdrawals queue
│       ├── users/+page.svelte        # User management
│       ├── liquidity/+page.svelte    # System liquidity monitor
│       ├── orders/+page.svelte       # All orders browser
│       ├── trades/+page.svelte       # All trades browser
│       └── audit/+page.svelte        # Admin audit log
├── docker-compose.yml                # Orchestrates all 5 services
├── .env.example                      # Copy to .env before deploy
└── README.md
```

---

## Database Schema

The full schema lives in [`backend/migrations/20240101000001_init.sql`](backend/migrations/20240101000001_init.sql). Highlights:

| Table | Purpose |
|---|---|
| `users` | User accounts with role (`user`/`admin`), status (active/suspended/banned/pending_kyc), KYC level, failed login counter |
| `wallets` | Per-user per-asset balances with separate `balance` and `locked_balance` columns |
| `orders` | Limit/market orders with side, type, price, quantity, filled_quantity, status |
| `trades` | Taker × maker cross-reference with taker_fee, maker_fee |
| `manual_transactions` | EGP deposits (fiat) + crypto withdrawals (crypto) with full lifecycle status |
| `wallet_ledger` | Append-only audit trail of every balance change |
| `admin_audit_log` | Every admin action (status change, manual tx review, etc.) |
| `system_liquidity` | Aggregate cache for admin overview |
| `settings` | JSON key/value store for fees, EGP/USD rate, trade pairs |

All monetary amounts use `NUMERIC(28, 8)` to preserve precision across both fiat (2dp) and crypto (8dp) values.

---

## Matching Engine

The engine is implemented in [`backend/src/matching_engine/mod.rs`](backend/src/matching_engine/mod.rs).

**Algorithm:**
- Each pair gets its own `OrderBook` guarded by a `parking_lot::RwLock`.
- Bids use `BTreeMap<PriceKey, VecDeque<Order>>` with reversed ordering (highest bid first).
- Asks use the same map with natural ordering (lowest ask first).
- `PriceKey(Decimal)` is a wrapper that provides `Ord` for decimal keys.
- When a new order arrives:
  1. Lock the opposite side's book.
  2. Walk best prices while price condition is satisfied.
  3. Match against the front of the VecDeque at each level (FIFO).
  4. If order is a limit and has remaining qty, insert it as a resting order.
- Trade events are emitted via `tokio::sync::mpsc` (for persistence) and `broadcast` (for WS fan-out).

**Complexity:**
- Insert: O(log n) — `BTreeMap` insertion
- Best-price lookup: O(1) — first/last key
- Match at price level: O(1) — VecDeque front
- Cancel: O(log n + k) — find level + linear scan within level (could be optimized with a secondary `HashMap<Uuid, (PriceKey, usize)>` index in production)

---

## Circuit Breaker

Implemented in [`backend/src/binance/mod.rs`](backend/src/binance/mod.rs).

**Trip conditions:**
1. No valid WebSocket message received for `CIRCUIT_BREAKER_TIMEOUT_SECS` (default 30s).
2. `CIRCUIT_BREAKER_MAX_FAILURES` consecutive parse/transport failures (default 5).

**Behavior when open:**
- All `POST /api/user/orders` requests return `503 SERVICE_UNAVAILABLE` with `code: "circuit_breaker_open"`.
- The user frontend shows a red banner and disables the order form.
- The admin overview displays "OPEN" status.
- The matching engine itself continues running — resting orders are still in memory, but no new taker orders can be submitted.

**Recovery:**
- A background watchdog checks every 2 seconds.
- When Binance feed recovers (first valid message received), the breaker closes automatically and trading resumes.

---

## Manual Transaction Flow

```
User submits deposit request
    │
    ▼
┌──────────────────────────────────────────┐
│ Backend inserts row in manual_transactions│
│ status = 'pending'                        │
│ Enqueues id in Redis list                 │
│ "egp_exchange:pending:deposit:fiat"       │
└──────────────────────────────────────────┘
    │
    ▼ (admin reviews)
┌──────────────────────────────────────────┐
│ Admin opens review modal                  │
│ - Verifies bank transfer receipt          │
│ - Sets status: under_review / approved /  │
│   completed / rejected / failed           │
│ - Optional admin_note                     │
└──────────────────────────────────────────┘
    │
    ▼ (admin finalizes)
┌──────────────────────────────────────────┐
│ If 'completed' (deposit):                 │
│   credit user EGP wallet                  │
│ If 'completed' (withdrawal):              │
│   unlock locked balance,                  │
│   debit user crypto wallet                │
│ If 'rejected'/'failed' (withdrawal):      │
│   unlock + refund locked balance          │
│ Publishes status to user's Redis channel  │
│ Dequeues id from pending list             │
│ Writes admin_audit_log entry              │
└──────────────────────────────────────────┘
```

Users see their queue position in real-time via the `queue_position` field returned by `GET /api/user/deposits` and `GET /api/user/withdrawals`.

---

## API Endpoints

### Public
| Method | Path | Description |
|---|---|---|
| GET | `/health` | Liveness probe |
| GET | `/health/ready` | Readiness probe (checks DB + Redis) |
| GET | `/api/market/tickers` | Latest ticker for all pairs |
| GET | `/api/market/orderbook/:pair` | Order book snapshot (depth=50) |
| GET | `/api/market/trades/:pair?limit=` | Recent public trades |
| GET | `/api/market/circuit` | Circuit breaker state |
| WS | `/api/market/ws?token=` | Multiplexed live stream (tickers, orderbook, trades, circuit events) |

### Auth
| Method | Path | Description |
|---|---|---|
| POST | `/api/auth/register` | Create user account |
| POST | `/api/auth/login` | Sign in (returns access + refresh JWT) |
| POST | `/api/auth/refresh` | Exchange refresh token for new pair |
| GET | `/api/user/me` | Current user profile |

### User (authenticated)
| Method | Path | Description |
|---|---|---|
| GET | `/api/user/wallets` | List user's wallets |
| GET/POST | `/api/user/orders` | List / place order |
| DELETE | `/api/user/orders/:id` | Cancel open order |
| GET | `/api/user/trades` | User's trade history |
| GET/POST | `/api/user/deposits` | List / request EGP deposit |
| GET/POST | `/api/user/withdrawals` | List / request crypto withdrawal |
| GET | `/api/user/ledger[/:asset]` | Wallet ledger entries |

### Admin (admin role required)
| Method | Path | Description |
|---|---|---|
| GET | `/api/admin/overview` | System-wide stats |
| GET | `/api/admin/users` | Paginated user list |
| GET | `/api/admin/users/:id` | User detail |
| PUT | `/api/admin/users/:id/status` | Update user status/KYC |
| GET | `/api/admin/manual_tx` | List manual transactions (filters: tx_type, status) |
| GET | `/api/admin/manual_tx/:id` | Manual tx detail |
| POST | `/api/admin/manual_tx/:id/review` | Approve / reject / complete manual tx |
| GET | `/api/admin/liquidity` | Aggregate liquidity per asset |
| GET | `/api/admin/orders` | All orders |
| GET | `/api/admin/trades` | All trades |
| GET | `/api/admin/audit` | Admin audit log |

---

## Quickstart (Docker Compose)

### Prerequisites
- Docker 24+
- Docker Compose v2.20+

### Steps

```bash
# 1. Clone and enter the project
cd egp-exchange

# 2. Generate a strong JWT secret
export JWT_SECRET=$(openssl rand -hex 32)

# 3. (Optional) customize environment
cp .env.example .env
# Edit .env to set ADMIN_BOOTSTRAP_EMAIL, frontend origins, etc.

# 4. Build and start all services
docker compose up -d --build

# 5. Watch the backend logs (migrations run automatically on first boot)
docker compose logs -f backend
```

Once healthy, open:
- **User frontend:** http://localhost:3000
- **Admin frontend:** http://localhost:3001
- **Backend health:** http://localhost:8080/health

Default admin credentials (change immediately in production):
```
Email: admin@egp-exchange.local
Password: ChangeMe!Admin2024
```

---

## Local Development (without Docker)

### Backend
```bash
cd backend
cp .env.example .env
# Edit .env: point DATABASE_URL and REDIS_URL to localhost services
# Start Postgres + Redis locally (e.g. via docker compose up postgres redis)

cargo sqlx migrate run --database-url "$DATABASE_URL"
cargo run
```

### Frontends
```bash
# Terminal 1 — user frontend
cd frontend-user
npm install
npm run dev

# Terminal 2 — admin frontend
cd frontend-admin
npm install
npm run dev
```

---

## Security Notes

1. **JWT secret** must be at least 32 characters. The backend refuses to start otherwise.
2. **Database password** and **Redis password** are baked into `docker-compose.yml` for dev convenience — replace them with secrets in production (use Docker secrets, Vault, or your orchestrator's secret manager).
3. **CORS** is restricted to the two configured frontend origins. Update `USER_FRONTEND_ORIGIN` and `ADMIN_FRONTEND_ORIGIN` for production domains.
4. **Circuit breaker** protects users from trading against stale prices. Tune `CIRCUIT_BREAKER_TIMEOUT_SECS` and `CIRCUIT_BREAKER_MAX_FAILURES` based on your latency tolerance.
5. **Admin endpoints** require an `admin` role JWT. The bootstrap admin is created on first launch with a default password — change it immediately.
6. **Wallet ledger** is append-only and provides a full audit trail. Never delete from it.
7. **Frontend isolation**: the user and admin SvelteKit apps are completely separate processes with no shared state. They can be deployed to different domains and even different networks.
8. **No third-party payment gateway**: EGP deposits/withdrawals are entirely manual. This is by design — it's a closed-loop system.

---

## Production Hardening Checklist

- [ ] Generate fresh `JWT_SECRET`, DB password, Redis password
- [ ] Set up a reverse proxy (Caddy / Nginx) with TLS termination for all three services
- [ ] Configure PostgreSQL with `pg_dump` automated backups
- [ ] Configure Redis with AOF persistence (already enabled in compose)
- [ ] Add rate limiting middleware to the backend (`tower::limit::ConcurrencyLimit` or `tower-governor`)
- [ ] Add Prometheus metrics exporter (via `tower-http::trace` + a `/metrics` endpoint)
- [ ] Run `cargo audit` regularly for vulnerable dependencies
- [ ] Run `cargo clippy -- -D warnings` in CI
- [ ] Set up Sentry / Loki for log aggregation
- [ ] Implement withdrawal address allow-listing per user (KYC-gated)
- [ ] Add 2FA for admin accounts
- [ ] Add IP allow-listing for admin frontend access (reverse proxy layer)
- [ ] Implement email/SMS notifications for deposit confirmations
- [ ] Conduct external security audit before going live

---

## License

Proprietary. See `LICENSE` file (if present) or contact the maintainers.
