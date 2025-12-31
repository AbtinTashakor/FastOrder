-- Add migration script here
ALTER TABLE carts
ALTER COLUMN user_id SET NOT NULL;

ALTER TABLE orders
ALTER COLUMN user_id SET NOT NULL;