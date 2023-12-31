CREATE TABLE posts (
  id INTEGER PRIMARY KEY,
  title TEXT NOT NULL,
  body TEXT NOT NULL,
  category_id INTEGER,
  author TEXT,
  published BOOLEAN NOT NULL DEFAULT 0,
  good_count INTEGER DEFAULT 0 NOT NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE category (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT
);
