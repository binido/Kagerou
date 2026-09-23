-- What the provider reports with each fetch: traffic used and allowed, the
-- end date, an announcement and a support link. NULL is "not reported",
-- which is also what every existing subscription holds until its next refresh.
ALTER TABLE sources ADD COLUMN traffic_used INTEGER;
ALTER TABLE sources ADD COLUMN traffic_total INTEGER;
ALTER TABLE sources ADD COLUMN expires_at INTEGER;
ALTER TABLE sources ADD COLUMN announce TEXT;
ALTER TABLE sources ADD COLUMN support_url TEXT;
