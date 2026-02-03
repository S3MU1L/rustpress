-- Deprecated (no-op).
--
-- The revision history schema is provided by:
--   - 20260202181000_content_item_revisions.sql
--   - 20260202181500_add_owner_kind_to_content_item_revisions.sql
--
-- This migration previously introduced an alternate schema. It is intentionally
-- left as a no-op to avoid breaking databases that already have the older
-- schema applied.

SELECT 1;
