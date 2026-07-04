-- ============================================================================
-- EGP Closed-Loop Cryptocurrency Exchange - Initial Schema
-- PostgreSQL 14+
-- ============================================================================
-- All monetary / crypto amounts use NUMERIC(28, 8) to preserve precision.
-- All timestamps are TIMESTAMPTZ stored as UTC.
-- ============================================================================

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ----------------------------------------------------------------------------
-- Enumerations
-- ----------------------------------------------------------------------------
CREATE TYPE user_role AS ENUM ('user', 'admin');
CREATE TYPE user_status AS ENUM ('active', 'suspended', 'banned', 'pending_kyc');
CREATE TYPE wallet_type AS ENUM ('fiat', 'crypto');
CREATE TYPE order_side AS ENUM ('buy', 'sell');
CREATE TYPE order_type AS ENUM ('limit', 'market');
CREATE TYPE order_status AS ENUM ('open', 'partially_filled', 'filled', 'cancelled', 'rejected');
CREATE TYPE manual_tx_type AS ENUM ('deposit', 'withdrawal');
CREATE TYPE manual_tx_asset_class AS ENUM ('fiat', 'crypto');
CREATE TYPE manual_tx_status AS ENUM ('pending', 'under_review', 'approved', 'rejected', 'completed', 'failed');
CREATE TYPE trade_side AS ENUM ('buy', 'sell');

-- ----------------------------------------------------------------------------
-- Users
-- ----------------------------------------------------------------------------
CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email           CITEXT UNIQUE NOT NULL,
    phone           TEXT UNIQUE,
    password_hash   TEXT NOT NULL,
    full_name       TEXT NOT NULL DEFAULT '',
    role            user_role NOT NULL DEFAULT 'user',
    status          user_status NOT NULL DEFAULT 'active',
    kyc_level       SMALLINT NOT NULL DEFAULT 0,
    country         TEXT NOT NULL DEFAULT 'EG',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login_at   TIMESTAMPTZ,
    failed_logins   SMALLINT NOT NULL DEFAULT 0,
    locked_until    TIMESTAMPTZ
);

CREATE INDEX idx_users_email ON users(lower(email));
CREATE INDEX idx_users_status ON users(status);
CREATE INDEX idx_users_role ON users(role);

-- ----------------------------------------------------------------------------
-- Wallets  (one row per (user_id, asset_symbol))
-- ----------------------------------------------------------------------------
CREATE TABLE wallets (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    asset_symbol    TEXT NOT NULL,                     -- e.g. 'EGP','BTC','ETH','USDT'
    wallet_type     wallet_type NOT NULL,
    balance         NUMERIC(28, 8) NOT NULL DEFAULT 0 CHECK (balance >= 0),
    locked_balance  NUMERIC(28, 8) NOT NULL DEFAULT 0 CHECK (locked_balance >= 0),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, asset_symbol)
);

CREATE INDEX idx_wallets_user ON wallets(user_id);
CREATE INDEX idx_wallets_asset ON wallets(asset_symbol);

-- ----------------------------------------------------------------------------
-- Orders
-- ----------------------------------------------------------------------------
CREATE TABLE orders (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    pair            TEXT NOT NULL,                     -- e.g. 'BTC_EGP'
    side            order_side NOT NULL,
    order_type      order_type NOT NULL,
    price           NUMERIC(28, 8),                    -- nullable for market orders
    quantity         NUMERIC(28, 8) NOT NULL CHECK (quantity > 0),
    filled_quantity  NUMERIC(28, 8) NOT NULL DEFAULT 0,
    status          order_status NOT NULL DEFAULT 'open',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_price_present_for_limit
        CHECK (order_type <> 'limit' OR (price IS NOT NULL AND price > 0)),
    CONSTRAINT chk_filled_le_quantity
        CHECK (filled_quantity <= quantity)
);

CREATE INDEX idx_orders_user ON orders(user_id);
CREATE INDEX idx_orders_pair_status ON orders(pair, status);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_created ON orders(created_at DESC);

-- ----------------------------------------------------------------------------
-- Trades (Taker × Maker cross-reference)
-- ----------------------------------------------------------------------------
CREATE TABLE trades (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pair            TEXT NOT NULL,
    taker_order_id  UUID NOT NULL REFERENCES orders(id) ON DELETE RESTRICT,
    maker_order_id  UUID NOT NULL REFERENCES orders(id) ON DELETE RESTRICT,
    taker_user_id   UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    maker_user_id   UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    taker_side      trade_side NOT NULL,
    price           NUMERIC(28, 8) NOT NULL CHECK (price > 0),
    quantity         NUMERIC(28, 8) NOT NULL CHECK (quantity > 0),
    taker_fee       NUMERIC(28, 8) NOT NULL DEFAULT 0,
    maker_fee       NUMERIC(28, 8) NOT NULL DEFAULT 0,
    executed_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_trades_pair ON trades(pair, executed_at DESC);
CREATE INDEX idx_trades_taker ON trades(taker_user_id, executed_at DESC);
CREATE INDEX idx_trades_maker ON trades(maker_user_id, executed_at DESC);
CREATE INDEX idx_trades_executed ON trades(executed_at DESC);

-- ----------------------------------------------------------------------------
-- Manual Transactions (EGP Deposits / Withdrawals + Crypto Withdrawals)
-- ----------------------------------------------------------------------------
CREATE TABLE manual_transactions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    tx_type         manual_tx_type NOT NULL,
    asset_class     manual_tx_asset_class NOT NULL,
    asset_symbol    TEXT NOT NULL,                     -- 'EGP' or crypto symbol
    amount          NUMERIC(28, 8) NOT NULL CHECK (amount > 0),
    fee             NUMERIC(28, 8) NOT NULL DEFAULT 0,
    status          manual_tx_status NOT NULL DEFAULT 'pending',
    -- For EGP: bank reference, receipt image url, agent note
    -- For Crypto: destination address, tx hash (when completed)
    reference       TEXT,                              -- user-supplied bank tx ref / address
    destination     TEXT,                              -- crypto withdrawal address
    tx_hash         TEXT,                              -- crypto on-chain hash (admin filled)
    receipt_url     TEXT,                              -- admin-uploaded receipt URL
    admin_note      TEXT,
    reviewed_by     UUID REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    reviewed_at     TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    CONSTRAINT chk_reference_xor_destination
        CHECK (
            (asset_class = 'fiat' AND reference IS NOT NULL) OR
            (asset_class = 'crypto' AND destination IS NOT NULL)
        )
);

CREATE INDEX idx_manual_tx_user ON manual_transactions(user_id, created_at DESC);
CREATE INDEX idx_manual_tx_status ON manual_transactions(status, created_at);
CREATE INDEX idx_manual_tx_type_status ON manual_transactions(tx_type, status);

-- ----------------------------------------------------------------------------
-- Wallet ledger (audit trail of every balance change)
-- ----------------------------------------------------------------------------
CREATE TABLE wallet_ledger (
    id              BIGSERIAL PRIMARY KEY,
    wallet_id       UUID NOT NULL REFERENCES wallets(id) ON DELETE RESTRICT,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    delta           NUMERIC(28, 8) NOT NULL,           -- can be negative
    balance_after   NUMERIC(28, 8) NOT NULL,
    reason          TEXT NOT NULL,                     -- 'trade','deposit','withdrawal','fee','adjustment'
    ref_id          UUID,                              -- order_id / manual_tx_id
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_ledger_wallet ON wallet_ledger(wallet_id, created_at DESC);
CREATE INDEX idx_ledger_user ON wallet_ledger(user_id, created_at DESC);
CREATE INDEX idx_ledger_ref ON wallet_ledger(ref_id);

-- ----------------------------------------------------------------------------
-- System Liquidity (admin-view aggregate cache, optional table)
-- ----------------------------------------------------------------------------
CREATE TABLE system_liquidity (
    asset_symbol    TEXT PRIMARY KEY,
    total_balance   NUMERIC(28, 8) NOT NULL DEFAULT 0,
    total_locked    NUMERIC(28, 8) NOT NULL DEFAULT 0,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ----------------------------------------------------------------------------
-- Settings (key/value) - used for things like fees, EGP/USD rate fallback
-- ----------------------------------------------------------------------------
CREATE TABLE settings (
    key             TEXT PRIMARY KEY,
    value           JSONB NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO settings(key, value) VALUES
    ('maker_fee_bps', '10'::jsonb),
    ('taker_fee_bps', '20'::jsonb),
    ('egp_usd_rate', '48.5'::jsonb),
    ('min_egp_deposit', '100'::jsonb),
    ('min_egp_withdrawal', '200'::jsonb),
    ('trade_pairs', '[{"pair":"BTC_EGP","base":"BTC","quote":"EGP","binance_symbol":"BTCUSDT"},{"pair":"ETH_EGP","base":"ETH","quote":"EGP","binance_symbol":"ETHUSDT"},{"pair":"USDT_EGP","base":"USDT","quote":"EGP","binance_symbol":"USDTUSDT"}]'::jsonb)
ON CONFLICT (key) DO NOTHING;

-- ----------------------------------------------------------------------------
-- Updated_at triggers
-- ----------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION trg_set_updated_at() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION trg_set_updated_at();
CREATE TRIGGER set_updated_at BEFORE UPDATE ON wallets
    FOR EACH ROW EXECUTE FUNCTION trg_set_updated_at();
CREATE TRIGGER set_updated_at BEFORE UPDATE ON orders
    FOR EACH ROW EXECUTE FUNCTION trg_set_updated_at();
CREATE TRIGGER set_updated_at BEFORE UPDATE ON system_liquidity
    FOR EACH ROW EXECUTE FUNCTION trg_set_updated_at();
CREATE TRIGGER set_updated_at BEFORE UPDATE ON settings
    FOR EACH ROW EXECUTE FUNCTION trg_set_updated_at();

-- ----------------------------------------------------------------------------
-- Audit log for admin actions
-- ----------------------------------------------------------------------------
CREATE TABLE admin_audit_log (
    id              BIGSERIAL PRIMARY KEY,
    admin_id        UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    action          TEXT NOT NULL,
    target_type     TEXT,
    target_id       UUID,
    details         JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_audit_admin ON admin_audit_log(admin_id, created_at DESC);
CREATE INDEX idx_audit_target ON admin_audit_log(target_type, target_id);
