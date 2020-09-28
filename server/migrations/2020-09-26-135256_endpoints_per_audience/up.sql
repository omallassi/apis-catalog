CREATE TABLE IF NOT EXISTS metrics_endpoints_per_audience (
    date_time TEXT NOT NULL UNIQUE, 
    data_points TEXT NOT NULL
);