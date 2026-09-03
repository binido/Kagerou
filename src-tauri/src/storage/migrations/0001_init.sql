CREATE TABLE sources (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    type         TEXT NOT NULL CHECK (type IN ('url', 'key')),
    value        TEXT NOT NULL,
    status       TEXT NOT NULL CHECK (status IN ('up-to-date', 'ready', 'refresh-due', 'updating')),
    last_refresh TEXT NOT NULL,
    origin_label TEXT NOT NULL
);

CREATE TABLE profile_groups (
    id        TEXT PRIMARY KEY,
    label     TEXT NOT NULL,
    kind      TEXT NOT NULL CHECK (kind IN ('default', 'custom', 'subscription')),
    source_id TEXT REFERENCES sources (id) ON DELETE SET NULL,
    is_open   INTEGER NOT NULL DEFAULT 1,
    position  INTEGER NOT NULL
);

CREATE TABLE profiles (
    id       TEXT PRIMARY KEY,
    name     TEXT NOT NULL,
    region   TEXT NOT NULL,
    protocol TEXT NOT NULL CHECK (protocol IN ('VLESS', 'VMess', 'Trojan', 'Shadowsocks', 'Hysteria2', 'Tuic')),
    origin   TEXT NOT NULL CHECK (origin IN ('local', 'imported')),
    group_id TEXT NOT NULL REFERENCES profile_groups (id) ON DELETE CASCADE,
    source_id TEXT REFERENCES sources (id) ON DELETE SET NULL,
    selected INTEGER NOT NULL DEFAULT 0,
    tcp_value TEXT NOT NULL,
    tcp_tone  TEXT NOT NULL CHECK (tcp_tone IN ('good', 'warn', 'bad', 'muted')),
    url_value TEXT NOT NULL,
    url_tone  TEXT NOT NULL CHECK (url_tone IN ('good', 'warn', 'bad', 'muted')),
    key      TEXT NOT NULL,
    position INTEGER NOT NULL
);

CREATE INDEX idx_profiles_group_id ON profiles (group_id);
CREATE INDEX idx_profiles_source_id ON profiles (source_id);

CREATE TABLE routing_presets (
    id          TEXT PRIMARY KEY,
    label       TEXT NOT NULL,
    description TEXT NOT NULL,
    enabled     INTEGER NOT NULL DEFAULT 0,
    position    INTEGER NOT NULL
);

CREATE TABLE routing_rules (
    id          TEXT PRIMARY KEY,
    match_value TEXT NOT NULL,
    outbound    TEXT NOT NULL CHECK (outbound IN ('Direct', 'Proxy', 'Block')),
    selected    INTEGER NOT NULL DEFAULT 0,
    position    INTEGER NOT NULL
);

CREATE TABLE settings (
    id                                  INTEGER PRIMARY KEY CHECK (id = 1),
    theme                               TEXT NOT NULL,
    language                            TEXT NOT NULL CHECK (language IN ('en', 'ru')),
    startup                             INTEGER NOT NULL,
    tun_interface                       TEXT NOT NULL,
    auto_update_subscriptions           INTEGER NOT NULL,
    subscription_update_interval        TEXT NOT NULL,
    custom_subscription_update_minutes  INTEGER NOT NULL,
    group_sort                          TEXT NOT NULL CHECK (group_sort IN ('ping', 'name', 'protocol'))
);

CREATE TABLE app_state (
    id                INTEGER PRIMARY KEY CHECK (id = 1),
    active_profile_id TEXT REFERENCES profiles (id) ON DELETE SET NULL
);

INSERT INTO settings (id, theme, language, startup, tun_interface, auto_update_subscriptions, subscription_update_interval, custom_subscription_update_minutes, group_sort)
VALUES (1, 'catppuccin-mocha', 'en', 1, 'utun / tun0', 0, '30', 60, 'ping');

INSERT INTO app_state (id, active_profile_id) VALUES (1, NULL);

-- Every install needs a default, non-deletable group for local
-- profiles/keys that don't belong to a subscription.
INSERT INTO profile_groups (id, label, kind, source_id, is_open, position)
VALUES ('default', 'Default', 'default', NULL, 1, 0);

INSERT INTO routing_presets (id, label, description, enabled, position) VALUES
    ('bypass-lan', 'Bypass LAN', 'Send local network traffic directly.', 1, 0),
    ('block-ads', 'Block ads', 'Block known advertising domains.', 1, 1);
