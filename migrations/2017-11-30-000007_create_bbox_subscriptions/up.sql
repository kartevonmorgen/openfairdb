CREATE TABLE bbox_subscriptions (
    id              TEXT PRIMARY KEY NOT NULL,
    south_west_lat  FLOAT NOT NULL,
    south_west_lng  FLOAT NOT NULL,
    north_east_lat  FLOAT NOT NULL,
    north_east_lng  FLOAT NOT NULL
);
