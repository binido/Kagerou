-- 2081 and 9091 belong to the latency-test core, which runs alongside the
-- connection. Keep them in step with app_state::AppState::new.
ALTER TABLE settings ADD COLUMN mixed_port INTEGER NOT NULL DEFAULT 2080
    CHECK (mixed_port BETWEEN 1 AND 65535 AND mixed_port NOT IN (2081, 9091));
ALTER TABLE settings ADD COLUMN clash_api_port INTEGER NOT NULL DEFAULT 9090
    CHECK (clash_api_port BETWEEN 1 AND 65535 AND clash_api_port NOT IN (2081, 9091)
        AND clash_api_port != mixed_port);
