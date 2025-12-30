-- =========================================
-- Daily order numbering (clean setup)
-- =========================================

-- 1) Table for daily counters (race-safe)
CREATE TABLE daily_counters (
    day DATE PRIMARY KEY,
    last_no INT NOT NULL
);

-- =========================================
-- 2) Orders table enhancements
-- =========================================

ALTER TABLE orders
    ADD COLUMN order_day DATE NOT NULL DEFAULT CURRENT_DATE,
    ADD COLUMN daily_no INT NOT NULL,
    ADD COLUMN order_code VARCHAR(20) NOT NULL;

-- =========================================
-- 3) Constraints & indexes
-- =========================================

-- Unique order number per day
CREATE UNIQUE INDEX idx_orders_order_day_daily_no
    ON orders (order_day, daily_no);

-- Unique human-readable order code
CREATE UNIQUE INDEX idx_orders_order_code
    ON orders (order_code);
