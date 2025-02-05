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
    yaw REAL NOT NULL,
    inventory_slot INTEGER NOT NULL
);

INSERT INTO player (x, y, z, roll, pitch, yaw, inventory_slot) VALUES (0, 5, 0, 0, 0, 0, 0);

CREATE TABLE inventory (
    item BLOB NOT NULL PRIMARY KEY,
    count INTEGER NOT NULL
);

CREATE TABLE hotbar (
    slot INTEGER NOT NULL PRIMARY KEY,
    item BLOB
);

INSERT INTO hotbar (slot, item) VALUES (0, NULL);
INSERT INTO hotbar (slot, item) VALUES (1, NULL);
INSERT INTO hotbar (slot, item) VALUES (2, NULL);
INSERT INTO hotbar (slot, item) VALUES (3, NULL);
INSERT INTO hotbar (slot, item) VALUES (4, NULL);
INSERT INTO hotbar (slot, item) VALUES (5, NULL);
INSERT INTO hotbar (slot, item) VALUES (6, NULL);
INSERT INTO hotbar (slot, item) VALUES (7, NULL);
INSERT INTO hotbar (slot, item) VALUES (8, NULL);
