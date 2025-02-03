-- Add migration script here
CREATE TABLE chunks (
    x INTEGER NOT NULL,
    y INTEGER NOT NULL,
    z INTEGER NOT NULL,
    data BLOB NOT NULL,
    PRIMARY KEY (x, y, z)
);

CREATE TABLE player (
    x REAL NOT NULL,
    y REAL NOT NULL,
    z REAL NOT NULL,
    roll REAL NOT NULL,
    pitch REAL NOT NULL,
    yaw REAL NOT NULL
);

INSERT INTO player (x, y, z, roll, pitch, yaw) VALUES (0, 5, 0, 0, 0, 0);
