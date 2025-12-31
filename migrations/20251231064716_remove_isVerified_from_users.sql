-- Add migration script here
ALTER TABLE users
DROP COLUMN IF EXISTS is_verified;