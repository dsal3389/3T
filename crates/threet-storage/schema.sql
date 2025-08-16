CREATE TABLE IF NOT EXISTS User (
    id INTEGER PRIMARY KEY,
    username TEXT,
    password TEXT
);

CREATE TABLE IF NOT EXISTS Channel (
    id INTEGER PRIMARY KEY,
    name TEXT
);
