-- The dashboard used to name the exit country by pulling a flag emoji out of
-- the profile's own name, which is a guess the subscription author made and
-- most names do not carry at all. It can now ask a public service, over the
-- tunnel, what the internet actually sees. That is a request to a third party
-- on every connection, so it is a setting rather than a fact of the app —
-- on by default, because a location nobody asked for is the feature.
ALTER TABLE settings ADD COLUMN geo_lookup INTEGER NOT NULL DEFAULT 1;
