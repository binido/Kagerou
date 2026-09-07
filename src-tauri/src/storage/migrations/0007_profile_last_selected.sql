-- The tray offers recently used profiles: a subscription of hundreds cannot
-- be a system menu, and nobody picks from hundreds anyway — people cycle
-- through the three or four that work for them.
ALTER TABLE profiles ADD COLUMN last_selected_at INTEGER;
