-- Add migration script here

ALTER TABLE carts
DROP CONSTRAINT IF EXISTS carts_status_check;


ALTER TABLE carts
ADD CONSTRAINT carts_status_check
CHECK (status IN ('active', 'confirming', 'locked'));

ALTER TABLE cart_items
ADD COLUMN IF NOT EXISTS price_snapshot BIGINT;

ALTER TABLE cart_items
ALTER COLUMN price_snapshot SET NOT NULL;

CREATE INDEX IF NOT EXISTS idx_cart_items_cart_id
ON cart_items(cart_id);
