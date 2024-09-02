CREATE TABLE IF NOT EXISTS monitor_ping (
    id INTEGER PRIMARY KEY,
    monitor_id INTEGER NOT NULL,
    status INTEGER NOT NULL,
    timestamp TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    bad INTEGER NOT NULL,
    FOREIGN KEY (monitor_id) REFERENCES monitor(id)
);
