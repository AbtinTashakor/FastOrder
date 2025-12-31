-- Add migration script here
ALTER TABLE carts
DROP CONSTRAINT IF EXISTS carts_customer_id_fkey;

ALTER TABLE orders
DROP CONSTRAINT IF EXISTS orders_customer_id_fkey;

-- حذف جدول legacy
DROP TABLE IF EXISTS customers;