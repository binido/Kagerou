-- Delay tests used to hardcode http://www.gstatic.com/generate_204. The URL
-- is now a setting next to the log level, in the Diagnostics section.
ALTER TABLE settings ADD COLUMN test_url TEXT NOT NULL DEFAULT 'http://www.gstatic.com/generate_204';
