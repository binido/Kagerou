-- TUN and system-proxy mode used to live only in the frontend store, where
-- they reset on every launch. They now sit in the settings page next to the
-- other preferences, so they persist like the rest of them.
ALTER TABLE settings ADD COLUMN tun_mode INTEGER NOT NULL DEFAULT 0;
ALTER TABLE settings ADD COLUMN system_proxy INTEGER NOT NULL DEFAULT 0;
