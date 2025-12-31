-- Add migration script here

ALTER TABLE carts
ADD COLUMN user_id UUID;

ALTER TABLE orders
ADD COLUMN user_id UUID;

-- FK
ALTER TABLE carts
ADD CONSTRAINT carts_user_id_fkey
FOREIGN KEY (user_id) REFERENCES users(id);

ALTER TABLE orders
ADD CONSTRAINT orders_user_id_fkey
FOREIGN KEY (user_id) REFERENCES users(id);
