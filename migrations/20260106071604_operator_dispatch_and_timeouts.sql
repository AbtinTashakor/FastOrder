-- =========================================
-- Operator dispatch + timeouts (future-proof)
-- =========================================

-- 0) Safety: extensions (already in init but harmless)
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- =========================================
-- 1) Operator state (only for users who have role=operator)
-- =========================================
CREATE TABLE IF NOT EXISTS operator_state (
    operator_id UUID PRIMARY KEY
        REFERENCES users(id)
        ON DELETE CASCADE,

    -- shift status
    is_on_shift BOOLEAN NOT NULL DEFAULT FALSE,

    -- UI state: LIST | ORDER
    current_view VARCHAR(16) NOT NULL DEFAULT 'LIST'
        CHECK (current_view IN ('LIST', 'ORDER')),

    -- if current_view=ORDER, which order is open
    current_order_id UUID NULL
        REFERENCES orders(id)
        ON DELETE SET NULL,

    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_operator_state_on_shift
    ON operator_state (is_on_shift);

-- =========================================
-- 2) Global system state (for round robin pointer, etc.)
-- =========================================
CREATE TABLE IF NOT EXISTS system_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- (optional) seed a placeholder key (safe to run multiple times)
INSERT INTO system_state(key, value)
VALUES ('last_operator_id', '')
ON CONFLICT (key) DO NOTHING;

-- =========================================
-- 3) Orders: new status model + assignment + time tracking + retries
-- =========================================

-- 3.1) Add new columns (nullable where needed)
ALTER TABLE orders
    ADD COLUMN IF NOT EXISTS operator_id UUID NULL
        REFERENCES users(id)
        ON DELETE SET NULL,

    ADD COLUMN IF NOT EXISTS assigned_at TIMESTAMPTZ NULL,
    ADD COLUMN IF NOT EXISTS seen_at TIMESTAMPTZ NULL,

    -- when customer clicks "repeat order"
    ADD COLUMN IF NOT EXISTS retry_count INT NOT NULL DEFAULT 0,

    -- optional: store reject reason for operator (UX later)
    ADD COLUMN IF NOT EXISTS rejection_reason TEXT NULL,

    -- optional: decision time for audit/debug
    ADD COLUMN IF NOT EXISTS decided_at TIMESTAMPTZ NULL;

CREATE INDEX IF NOT EXISTS idx_orders_operator_id
    ON orders (operator_id);

CREATE INDEX IF NOT EXISTS idx_orders_assigned_at
    ON orders (assigned_at);

-- 3.2) Migrate old lowercase statuses -> new uppercase statuses
-- (If you already have rows, this keeps them valid.)
UPDATE orders
SET status = 'PENDING_ASSIGN'
WHERE status = 'pending';

UPDATE orders
SET status = 'ACCEPTED'
WHERE status = 'accepted';

UPDATE orders
SET status = 'REJECTED'
WHERE status = 'rejected';

-- 3.3) Replace old status check constraint with new one
-- Default unnamed check constraint usually becomes "orders_status_check"
ALTER TABLE orders
    DROP CONSTRAINT IF EXISTS orders_status_check;

ALTER TABLE orders
    ADD CONSTRAINT orders_status_check CHECK (
        status IN (
            'PENDING_ASSIGN',
            'ASSIGNED_UNSEEN',
            'ASSIGNED_IN_REVIEW',
            'WAITING_FOR_OPERATOR',
            'ACCEPTED',
            'REJECTED'
        )
    );

-- 3.4) Ensure a sensible default for new orders
ALTER TABLE orders
    ALTER COLUMN status SET DEFAULT 'PENDING_ASSIGN';

-- =========================================
-- 4) Optional: helpful composite index for "pending work queue"
-- =========================================
CREATE INDEX IF NOT EXISTS idx_orders_status_created_at
    ON orders (status, created_at);

-- Done
