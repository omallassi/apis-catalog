extern crate rusqlite;

use chrono::{DateTime, Utc};

use rusqlite::{params, Connection, Result};
use rusqlite::{NO_PARAMS, named_params};

use log::{debug, info};

use super::super::settings::{*};

pub fn save_metrics_pull_requests_number(config:  &super::super::settings::Database, datetime: DateTime<Utc>, size: i32) -> Result<()> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-metrics.db");
    {
        debug!("Saving metrics into Metrics_Database [{:?}]", db_path);
    }

    let conn = Connection::open(db_path)?;
    conn.execute("CREATE TABLE IF NOT EXISTS metrics (
            date_time TEXT NOT NULL UNIQUE, 
            value INTEGER NOT NULL
        )", 
        NO_PARAMS,
    )?;

    conn.execute(
        "INSERT INTO metrics (date_time, value) VALUES (?1, ?2)", 
        params![datetime, size],
    )?;

    Ok(())
}

#[derive(Debug)]
pub struct TimeSeries {
    points : Vec<(DateTime<Utc>, i32)>,
}

pub fn get_metrics_pull_requests_number(config:  &super::super::settings::Database) -> Result<TimeSeries> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-metrics.db");
    {
        debug!("Reading all [pull_requests_number] metrics into Metrics_Database [{:?}]", db_path);
    }

    let conn = Connection::open(db_path)?;
    conn.execute("CREATE TABLE IF NOT EXISTS metrics (
            date_time TEXT NOT NULL UNIQUE, 
            value INTEGER NOT NULL
        )", 
        NO_PARAMS,
    )?;

    let mut stmt = conn.prepare("SELECT date_time, value FROM metrics")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut points = Vec::new();
    while let Some(row) = rows.next()? {
        let time = row.get("date_time")?;
        let val = row.get("value")?;

        points.push((time, val));
    }

    let timeseries = TimeSeries {
        points : points,
    };

    Ok(timeseries)
}