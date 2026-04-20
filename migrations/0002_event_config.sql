-- ─────────────────────────────────────────────
-- EVENT CONFIG
-- Singleton row (id = 1) with editable app-level settings.
-- ─────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS event_config (
    id           INTEGER PRIMARY KEY CHECK (id = 1),
    event_name   TEXT    NOT NULL DEFAULT 'Evento',
    updated_at   INTEGER NOT NULL DEFAULT (unixepoch())
);

INSERT OR IGNORE INTO event_config (id, event_name) VALUES (1, 'Evento');
