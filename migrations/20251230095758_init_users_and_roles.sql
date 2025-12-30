-- Add migration script here
/* ================================
   Roles
================================ */
CREATE TABLE roles (
    id SMALLSERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    description TEXT
);

/* ================================
   Users
================================ */
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- identity
    telegram_id BIGINT UNIQUE,
    telegram_username VARCHAR(64),
    phone VARCHAR(20),

    -- profile
    full_name VARCHAR(150),

    -- status
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,

    -- timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

/* ================================
   Indexes
================================ */
CREATE INDEX idx_users_telegram_id ON users(telegram_id);
CREATE INDEX idx_users_phone ON users(phone);
CREATE INDEX idx_users_active ON users(is_active);

/* ================================
   User Roles (RBAC)
================================ */
CREATE TABLE user_roles (
    user_id UUID NOT NULL
        REFERENCES users(id)
        ON DELETE CASCADE,

    role_id SMALLINT NOT NULL
        REFERENCES roles(id)
        ON DELETE RESTRICT,

    assigned_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (user_id, role_id)
);

/* ================================
   Seed Roles
================================ */
INSERT INTO roles (name, description) VALUES
('admin', 'System administrator'),
('operator', 'Order operator'),
('customer', 'End customer');
