-- Auto-connect on launch did not exist: the app started disconnected until
-- the user pressed connect. It is now a stored setting, off by default —
-- an app that dials out on its own without being asked is the kind of
-- thing people uninstall over.
ALTER TABLE settings ADD COLUMN auto_connect INTEGER NOT NULL DEFAULT 0;
