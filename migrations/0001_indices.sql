-- Performance indices for nyobarust
-- Run via: psql $DATABASE_URL -f migrations/0001_indices.sql
-- All IF NOT EXISTS; safe to re-run.

-- users: login lookup (case-sensitive equality on username)
CREATE UNIQUE INDEX IF NOT EXISTS users_username_uidx ON users (username);

-- brands: hot join on owner_id, owner scope filter
CREATE INDEX IF NOT EXISTS brands_owner_id_idx ON brands (owner_id) WHERE deleted_at IS NULL;

-- parfume
CREATE INDEX IF NOT EXISTS parfume_brands_id_idx ON parfume (brands_id) WHERE deleted_at IS NULL;

-- batch_parfume
CREATE INDEX IF NOT EXISTS batch_parfume_parfume_id_idx ON batch_parfume (parfume_id) WHERE deleted_at IS NULL;

-- batch_parfume_bottle: join from bottle to batch
CREATE INDEX IF NOT EXISTS batch_parfume_bottle_batch_id_idx ON batch_parfume_bottle (batch_parfume_id) WHERE deleted_at IS NULL;

-- decant
CREATE INDEX IF NOT EXISTS decant_parfume_id_idx ON decant (parfume_id) WHERE deleted_at IS NULL;

-- order_items: hottest table. bottle_id joins history/ranking.
CREATE INDEX IF NOT EXISTS order_items_bottle_id_idx ON order_items (bottle_id);
CREATE INDEX IF NOT EXISTS order_items_decant_id_idx ON order_items (decant_id);

-- success-only time range for revenue/ranking (most reads filter status='success')
CREATE INDEX IF NOT EXISTS order_items_success_created_at_idx
    ON order_items (created_at DESC)
    WHERE status = 'success';

-- stock_movements: EXISTS checks on soft-delete and ledger queries
CREATE INDEX IF NOT EXISTS stock_movements_bottle_id_idx ON stock_movements (bottle_id);
CREATE INDEX IF NOT EXISTS stock_movements_order_items_id_idx ON stock_movements (order_items_id)
    WHERE order_items_id IS NOT NULL;

-- refresh_token: family_id lookup is hot path on login + refresh
CREATE INDEX IF NOT EXISTS refresh_token_family_id_idx ON refresh_token (family_id);
CREATE INDEX IF NOT EXISTS refresh_token_user_id_idx ON refresh_token (user_id);

-- ANALYZE so planner picks up new stats
ANALYZE users;
ANALYZE brands;
ANALYZE parfume;
ANALYZE batch_parfume;
ANALYZE batch_parfume_bottle;
ANALYZE decant;
ANALYZE order_items;
ANALYZE stock_movements;
ANALYZE refresh_token;
