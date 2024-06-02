CREATE TABLE IF NOT EXISTS monitor_stats (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  average_response_ms INTEGER NOT NULL,
  uptime_percentage_24h INTEGER NOT NULL,
  uptime_percentage_30d INTEGER NOT NULL
);
