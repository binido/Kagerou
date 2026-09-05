-- `startup` was seeded on, back when nothing acted on it. It now registers a
-- real login item, and an app that adds itself to login items on first run
-- without being asked is the kind of thing people uninstall over.
--
-- Existing rows are reset too, deliberately: the setting was inert in every
-- released build, so a stored 1 records nobody's decision. Turning it off
-- discards no intent.
UPDATE settings SET startup = 0 WHERE id = 1;
