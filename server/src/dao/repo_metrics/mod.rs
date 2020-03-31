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
        debug!("Saving metrics_pr_num into Metrics_Database [{:?}]", db_path);
    }

    let conn = Connection::open(db_path)?;
    conn.execute("CREATE TABLE IF NOT EXISTS metrics_pr_num (
            date_time TEXT NOT NULL UNIQUE, 
            value INTEGER NOT NULL
        )", 
        NO_PARAMS,
    )?;

    conn.execute(
        "INSERT INTO metrics_pr_num (date_time, value) VALUES (?1, ?2)", 
        params![datetime, size],
    )?;

    Ok(())
}

#[derive(Debug)]
pub struct TimeSeries {
    pub points : Vec<(DateTime<Utc>, i32)>,
}

pub fn get_metrics_pull_requests_number(config:  &super::super::settings::Database) -> Result<TimeSeries> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-metrics.db");
    {
        debug!("Reading all [pull_requests_number] metrics into Metrics_Database [{:?}]", db_path);
    }

    let conn = Connection::open(db_path)?;
    conn.execute("CREATE TABLE IF NOT EXISTS metrics_pr_num (
            date_time TEXT NOT NULL UNIQUE, 
            value INTEGER NOT NULL
        )", 
        NO_PARAMS,
    )?;

    let mut stmt = conn.prepare("SELECT date_time, value FROM metrics_pr_num")?;
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

pub fn save_metrics_pull_requests_ages(config:  &super::super::settings::Database, datetime: DateTime<Utc>, p0: isize, p50: isize, p100: isize, mean: isize) -> Result<()> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-metrics.db");
    {
        debug!("Saving metrics_pr_ages into Metrics_Database [{:?}]", db_path);
    }

    let conn = Connection::open(db_path)?;
    conn.execute("CREATE TABLE IF NOT EXISTS metrics_pr_ages (
            date_time TEXT NOT NULL UNIQUE, 
            p0 INTEGER NOT NULL, 
            p50 INTEGER NOT NULL, 
            p100 INTEGER NOT NULL, 
            mean INTEGER NOT NULL
        )", 
        NO_PARAMS
    )?;

    conn.execute(
        "INSERT INTO metrics_pr_ages (date_time, p0, p50, p100, mean) VALUES (?1, ?2, ?3, ?4, ?5)", 
        params![datetime, p0, p50, p100, mean],
    )?;

    Ok(())
}

#[derive(Debug)]
pub struct TupleTimeSeries {
    pub points : Vec<(DateTime<Utc>, i64, i64, i64, i64)>,
}

pub fn get_metrics_pull_requests_ages(config:  &super::super::settings::Database) -> Result<TupleTimeSeries> { 
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-metrics.db");
    {
        debug!("Reading all [pull_requests_ages] metrics into Metrics_Database [{:?}]", db_path);
    }

    let conn = Connection::open(db_path)?;
    conn.execute("CREATE TABLE IF NOT EXISTS metrics_pr_ages (
            date_time TEXT NOT NULL UNIQUE, 
            p0 INTEGER NOT NULL, 
            p50 INTEGER NOT NULL, 
            p100 INTEGER NOT NULL, 
            mean INTEGER NOT NULL
        )", 
        NO_PARAMS
    )?;

    let mut stmt = conn.prepare("SELECT date_time, p0, p50, p100, mean FROM metrics_pr_ages")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut points = Vec::new();
    while let Some(row) = rows.next()? {
        points.push((row.get("date_time")?, row.get("p0")?, row.get("p50")?, row.get("p100")?, row.get("mean")?));
    }

    let timeseries = TupleTimeSeries {
        points : points,
    };

    Ok(timeseries)
}

pub fn save_metrics_endpoints_num(config:  &super::super::settings::Database, datetime: DateTime<Utc>, size: i32) -> Result<()> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-metrics.db");
    {
        debug!("Saving metrics_endpoints_num into Metrics_Database [{:?}]", db_path);
    }

    let conn = Connection::open(db_path)?;
    conn.execute("CREATE TABLE IF NOT EXISTS metrics_endpoints_num (
            date_time TEXT NOT NULL UNIQUE, 
            value INTEGER NOT NULL
        )", 
        NO_PARAMS,
    )?;

    conn.execute(
        "INSERT INTO metrics_endpoints_num (date_time, value) VALUES (?1, ?2)", 
        params![datetime, size],
    )?;

    Ok(())
}

pub fn get_metrics_endpoints_number(config:  &super::super::settings::Database) -> Result<TimeSeries> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-metrics.db");
    {
        debug!("Reading all [metrics_endpoints_num] metrics into Metrics_Database [{:?}]", db_path);
    }

    let conn = Connection::open(db_path)?;
    conn.execute("CREATE TABLE IF NOT EXISTS metrics_endpoints_num (
            date_time TEXT NOT NULL UNIQUE, 
            value INTEGER NOT NULL
        )", 
        NO_PARAMS,
    )?;

    let mut stmt = conn.prepare("SELECT date_time, value FROM metrics_endpoints_num")?;
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