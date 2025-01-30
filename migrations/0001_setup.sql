-- Add migration script here
CREATE TABLE chunks (
    x INTEGER NOT NULL,
    y INTEGER NOT NULL,
    z INTEGER NOT NULL,
    data BLOB NOT NULL,
    PRIMARY KEY (x, y, z)
);
