-- The log level used to be hardcoded to "info" in the generated config.
-- NekoBox exposes it in settings, and so do we now, in a Diagnostics section.
ALTER TABLE settings ADD COLUMN log_level TEXT NOT NULL DEFAULT 'info';
