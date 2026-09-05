-- The log level used to be hardcoded to "info" in the generated config.
-- NekoBox exposes it in settings, and so do we now, in a Diagnostics section.
ALTER TABLE settings ADD COLUMN log_level TEXT NOT NULL DEFAULT 'info'
    CHECK (log_level IN ('trace', 'debug', 'info', 'warn', 'error', 'fatal', 'panic'));
