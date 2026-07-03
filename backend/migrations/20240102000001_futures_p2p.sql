-- ============================================================================
-- إضافة العقود الآجلة + التداول بين الأفراد + إعدادات العملات
-- Add Futures + P2P + Currency settings
-- ============================================================================

-- ----------------------------------------------------------------------------
-- العقود الآجلة - Futures positions
-- ----------------------------------------------------------------------------
CREATE TYPE position_side AS ENUM ('long', 'short');
CREATE TYPE margin_mode AS ENUM ('isolated', 'cross');
CREATE TYPE position_status AS ENUM ('open', 'closed', 'liquidated');

CREATE TABLE futures_positions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    pair                TEXT NOT NULL,
    side                position_side NOT NULL,
    margin_mode         margin_mode NOT NULL DEFAULT 'isolated',
    leverage            INTEGER NOT NULL CHECK (leverage >= 1 AND leverage <= 125),
    margin              NUMERIC(28, 8) NOT NULL CHECK (margin > 0),
    quantity             NUMERIC(28, 8) NOT NULL CHECK (quantity > 0),
    entry_price         NUMERIC(28, 8) NOT NULL,
    mark_price          NUMERIC(28, 8) NOT NULL,
    liquidation_price   NUMERIC(28, 8) NOT NULL,
    unrealized_pnl      NUMERIC(28, 8) NOT NULL DEFAULT 0,
    realized_pnl        NUMERIC(28, 8) NOT NULL DEFAULT 0,
    status              position_status NOT NULL DEFAULT 'open',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    closed_at           TIMESTAMPTZ,
    close_price         NUMERIC(28, 8),
    CONSTRAINT chk_futures_margin CHECK (margin > 0)
);

CREATE INDEX idx_futures_user ON futures_positions(user_id, status);
CREATE INDEX idx_futures_status ON futures_positions(status);
CREATE INDEX idx_futures_pair ON futures_positions(pair, status);

-- ----------------------------------------------------------------------------
-- مدفوعات التمويل - Funding payments
-- ----------------------------------------------------------------------------
CREATE TABLE funding_payments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    position_id     UUID NOT NULL REFERENCES futures_positions(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    pair            TEXT NOT NULL,
    amount          NUMERIC(28, 8) NOT NULL,  -- موجب = يدفع للمركز، سالب = يستلم
    funding_rate    NUMERIC(28, 8) NOT NULL,
    paid_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_funding_position ON funding_payments(position_id, paid_at DESC);

-- ----------------------------------------------------------------------------
-- تصفية المراكز - Liquidations
-- ----------------------------------------------------------------------------
CREATE TABLE liquidations (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    position_id     UUID NOT NULL REFERENCES futures_positions(id) ON DELETE RESTRICT,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    pair            TEXT NOT NULL,
    liquidation_price   NUMERIC(28, 8) NOT NULL,
    mark_price      NUMERIC(28, 8) NOT NULL,
    realized_pnl    NUMERIC(28, 8) NOT NULL,
    fee             NUMERIC(28, 8) NOT NULL DEFAULT 0,
    liquidated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_liquidations_user ON liquidations(user_id, liquidated_at DESC);

-- ----------------------------------------------------------------------------
-- إعدادات المستخدم للعقود الآجلة - Per-user futures settings
-- ----------------------------------------------------------------------------
CREATE TABLE futures_user_settings (
    user_id         UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    pair            TEXT NOT NULL,
    leverage        INTEGER NOT NULL DEFAULT 10,
    margin_mode     margin_mode NOT NULL DEFAULT 'isolated',
    UNIQUE (user_id, pair)
);

-- ----------------------------------------------------------------------------
-- التداول بين الأفراد - P2P offers
-- ----------------------------------------------------------------------------
CREATE TYPE p2p_side AS ENUM ('buy', 'sell');
CREATE TYPE p2p_offer_status AS ENUM ('active', 'paused', 'closed');
CREATE TYPE p2p_trade_status AS ENUM ('pending', 'paid', 'released', 'cancelled', 'disputed', 'completed');

CREATE TABLE p2p_offers (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    side                p2p_side NOT NULL,
    asset_symbol        TEXT NOT NULL,
    price_margin_pct    NUMERIC(8, 4) NOT NULL DEFAULT 0,
    min_amount_egp      NUMERIC(28, 8) NOT NULL CHECK (min_amount_egp > 0),
    max_amount_egp      NUMERIC(28, 8) NOT NULL CHECK (max_amount_egp > 0),
    payment_methods     TEXT[] NOT NULL,
    time_limit_min      INTEGER NOT NULL DEFAULT 30,
    status              p2p_offer_status NOT NULL DEFAULT 'active',
    total_trades        INTEGER NOT NULL DEFAULT 0,
    completion_rate     NUMERIC(5, 2) NOT NULL DEFAULT 100.00,
    avg_release_min     INTEGER NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_p2p_max_gt_min CHECK (max_amount_egp >= min_amount_egp)
);

CREATE INDEX idx_p2p_offers_status ON p2p_offers(status, side);
CREATE INDEX idx_p2p_offers_asset ON p2p_offers(asset_symbol, status);
CREATE INDEX idx_p2p_offers_user ON p2p_offers(user_id);

-- ----------------------------------------------------------------------------
-- صفقات P2P - P2P trades
-- ----------------------------------------------------------------------------
CREATE TABLE p2p_trades (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    offer_id            UUID NOT NULL REFERENCES p2p_offers(id) ON DELETE RESTRICT,
    buyer_id            UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    seller_id           UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    asset_symbol        TEXT NOT NULL,
    amount              NUMERIC(28, 8) NOT NULL CHECK (amount > 0),
    price_egp           NUMERIC(28, 8) NOT NULL CHECK (price_egp > 0),
    total_egp           NUMERIC(28, 8) NOT NULL,
    payment_method      TEXT NOT NULL,
    status              p2p_trade_status NOT NULL DEFAULT 'pending',
    escrow_locked       BOOLEAN NOT NULL DEFAULT true,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    paid_at             TIMESTAMPTZ,
    released_at         TIMESTAMPTZ,
    cancelled_at        TIMESTAMPTZ,
    completed_at        TIMESTAMPTZ,
    CONSTRAINT chk_p2p_trade_parties CHECK (buyer_id <> seller_id)
);

CREATE INDEX idx_p2p_trades_buyer ON p2p_trades(buyer_id, created_at DESC);
CREATE INDEX idx_p2p_trades_seller ON p2p_trades(seller_id, created_at DESC);
CREATE INDEX idx_p2p_trades_status ON p2p_trades(status, created_at);
CREATE INDEX idx_p2p_trades_offer ON p2p_trades(offer_id);

-- ----------------------------------------------------------------------------
-- رسائل محادثة الصفقات - P2P chat messages
-- ----------------------------------------------------------------------------
CREATE TABLE p2p_messages (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trade_id        UUID NOT NULL REFERENCES p2p_trades(id) ON DELETE CASCADE,
    sender_id       UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    message         TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_p2p_messages_trade ON p2p_messages(trade_id, created_at);

-- ----------------------------------------------------------------------------
-- تقييمات P2P - P2P reviews
-- ----------------------------------------------------------------------------
CREATE TABLE p2p_reviews (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trade_id        UUID NOT NULL REFERENCES p2p_trades(id) ON DELETE CASCADE,
    reviewer_id     UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    reviewed_id     UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    rating          SMALLINT NOT NULL CHECK (rating >= 1 AND rating <= 5),
    comment         TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (trade_id, reviewer_id)
);

-- ----------------------------------------------------------------------------
-- العملات المدعومة - Supported currencies (تحكم الأدمن)
-- ----------------------------------------------------------------------------
CREATE TYPE currency_type AS ENUM ('fiat', 'crypto');

CREATE TABLE currencies (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol          TEXT UNIQUE NOT NULL,
    name            TEXT NOT NULL,
    type            currency_type NOT NULL,
    precision       SMALLINT NOT NULL DEFAULT 8,
    withdraw_fee    NUMERIC(28, 8) NOT NULL DEFAULT 0,
    min_withdrawal  NUMERIC(28, 8) NOT NULL DEFAULT 0,
    network         TEXT,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_currencies_type ON currencies(type, is_active);

-- ----------------------------------------------------------------------------
-- أزواج التداول - Trading pairs (تحكم شامل للأدمن)
-- ----------------------------------------------------------------------------
CREATE TABLE trading_pairs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pair            TEXT UNIQUE NOT NULL,
    base_asset      TEXT NOT NULL,
    quote_asset     TEXT NOT NULL,
    binance_symbol  TEXT NOT NULL,
    is_spot_active  BOOLEAN NOT NULL DEFAULT true,
    is_futures_active BOOLEAN NOT NULL DEFAULT false,
    maker_fee_bps   INTEGER NOT NULL DEFAULT 10,
    taker_fee_bps   INTEGER NOT NULL DEFAULT 20,
    min_order_qty   NUMERIC(28, 8) NOT NULL DEFAULT 0.0001,
    price_precision SMALLINT NOT NULL DEFAULT 2,
    qty_precision   SMALLINT NOT NULL DEFAULT 8,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_trading_pairs_active ON trading_pairs(is_active, sort_order);

-- ----------------------------------------------------------------------------
-- إدراج البيانات الأولية - Seed data
-- ----------------------------------------------------------------------------
INSERT INTO currencies (symbol, name, type, precision, withdraw_fee, min_withdrawal, network, is_active) VALUES
    ('EGP', 'الجنيه المصري', 'fiat', 2, 0, 100, NULL, true),
    ('BTC', 'بيتكوين', 'crypto', 8, 0.0005, 0.001, 'Bitcoin', true),
    ('ETH', 'إيثيريوم', 'crypto', 8, 0.005, 0.01, 'ERC-20', true),
    ('USDT', 'تيثر', 'crypto', 8, 1, 10, 'TRC-20', true),
    ('USDC', 'USD Coin', 'crypto', 8, 1, 10, 'ERC-20', true),
    ('BNB', 'BNB', 'crypto', 8, 0.005, 0.01, 'BSC', true)
ON CONFLICT (symbol) DO NOTHING;

INSERT INTO trading_pairs (pair, base_asset, quote_asset, binance_symbol, is_spot_active, is_futures_active, maker_fee_bps, taker_fee_bps, sort_order) VALUES
    ('BTC_EGP', 'BTC', 'EGP', 'BTCUSDT', true, true, 10, 20, 1),
    ('ETH_EGP', 'ETH', 'EGP', 'ETHUSDT', true, true, 10, 20, 2),
    ('USDT_EGP', 'USDT', 'EGP', 'USDTUSDT', true, false, 5, 10, 3)
ON CONFLICT (pair) DO NOTHING;

-- ----------------------------------------------------------------------------
-- محفزات updated_at - Updated_at triggers
-- ----------------------------------------------------------------------------
CREATE TRIGGER set_updated_at BEFORE UPDATE ON p2p_offers
    FOR EACH ROW EXECUTE FUNCTION trg_set_updated_at();
CREATE TRIGGER set_updated_at BEFORE UPDATE ON trading_pairs
    FOR EACH ROW EXECUTE FUNCTION trg_set_updated_at();
CREATE TRIGGER set_updated_at BEFORE UPDATE ON currencies
    FOR EACH ROW EXECUTE FUNCTION trg_set_updated_at();
