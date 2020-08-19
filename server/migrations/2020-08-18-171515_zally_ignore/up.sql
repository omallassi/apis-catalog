CREATE TABLE IF NOT EXISTS metrics_zally_ignore (
    date_time TEXT NOT NULL UNIQUE, 
    data_points TEXT NOT NULL
);