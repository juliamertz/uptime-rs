CREATE TABLE IF NOT EXISTS monitor (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  ip TEXT NOT NULL,
  port INTEGER,
  interval INTEGER NOT NULL,
  paused INTEGER NOT NULL
);
