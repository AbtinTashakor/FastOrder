-- Enable UUID generation
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- customers (whitelist-based)
CREATE TABLE customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phone_number VARCHAR(20) NOT NULL UNIQUE,
    telegram_user_id BIGINT UNIQUE,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- menu categories
CREATE TABLE menu_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(100) NOT NULL,
    position INT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

-- menu items
CREATE TABLE menu_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category_id UUID NOT NULL REFERENCES menu_categories(id),
    title VARCHAR(150) NOT NULL,
    price BIGINT NOT NULL,
    position INT NOT NULL DEFAULT 0,
    is_available BOOLEAN NOT NULL DEFAULT TRUE
);

-- carts
CREATE TABLE carts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES customers(id),
    status VARCHAR(20) NOT NULL CHECK (status IN ('active', 'locked')),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- cart items
CREATE TABLE cart_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cart_id UUID NOT NULL REFERENCES carts(id) ON DELETE CASCADE,
    menu_item_id UUID NOT NULL REFERENCES menu_items(id),
    quantity INT NOT NULL CHECK (quantity > 0),
    UNIQUE (cart_id, menu_item_id)
);

-- orders
CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES customers(id),
    total_price BIGINT NOT NULL,
    status VARCHAR(20) NOT NULL CHECK (
        status IN ('pending', 'accepted', 'rejected')
    ),
    prep_time_minutes INT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- order items (snapshot)
CREATE TABLE order_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    title_snapshot VARCHAR(150) NOT NULL,
    price_snapshot BIGINT NOT NULL,
    quantity INT NOT NULL CHECK (quantity > 0)
);

-- indexes
CREATE INDEX idx_customers_phone ON customers(phone_number);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_menu_items_category ON menu_items(category_id);
