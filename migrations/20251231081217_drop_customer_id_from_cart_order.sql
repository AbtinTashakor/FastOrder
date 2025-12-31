-- Add migration script here
ALTER TABLE carts
DROP CONSTRAINT IF EXISTS carts_customer_id_fkey;

ALTER TABLE orders
DROP CONSTRAINT IF EXISTS orders_customer_id_fkey;

ALTER TABLE carts
DROP COLUMN customer_id;

ALTER TABLE orders
DROP COLUMN customer_id;