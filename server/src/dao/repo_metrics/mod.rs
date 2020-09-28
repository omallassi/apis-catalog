extern crate rusqlite;

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use rusqlite::NO_PARAMS;
use rusqlite::{params, Connection, Result};

use log::{debug, info, warn};

pub fn save_metrics_pull_requests_number(
    config: &super::super::settings::Database,
    datetime: DateTime<Utc>,
    size: i32,
) -> Result<()> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!(
            "Saving metrics_pr_num into Metrics_Database [{:?}]",
            db_path
        );
    }

    let conn = Connection::open(db_path)?;
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS metrics_pr_num (
    //         date_time TEXT NOT NULL UNIQUE,
    //         value INTEGER NOT NULL
    //     )",
    //     NO_PARAMS,
    // )?;

    conn.execute(
        "INSERT INTO metrics_pr_num (date_time, value) VALUES (?1, ?2)",
        params![datetime, size],
    )?;

    Ok(())
}

#[derive(Debug)]
pub struct TimeSeries {
    pub points: Vec<(DateTime<Utc>, i32)>,
}

pub fn get_metrics_pull_requests_number(
    config: &super::super::settings::Database,
) -> Result<TimeSeries> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!(
            "Reading all [pull_requests_number] metrics into Metrics_Database [{:?}]",
            db_path
        );
    }

    let conn = Connection::open(db_path)?;
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS metrics_pr_num (
    //         date_time TEXT NOT NULL UNIQUE,
    //         value INTEGER NOT NULL
    //     )",
    //     NO_PARAMS,
    // )?;

    let mut stmt = conn.prepare("SELECT date_time, value FROM metrics_pr_num")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut points = Vec::new();
    while let Some(row) = rows.next()? {
        let time = row.get("date_time")?;
        let val = row.get("value")?;

        points.push((time, val));
    }

    let timeseries = TimeSeries { points: points };

    Ok(timeseries)
}

pub fn save_metrics_pull_requests_ages(
    config: &super::super::settings::Database,
    datetime: DateTime<Utc>,
    p0: isize,
    p50: isize,
    p100: isize,
    mean: isize,
) -> Result<()> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!(
            "Saving metrics_pr_ages into Metrics_Database [{:?}]",
            db_path
        );
    }

    let conn = Connection::open(db_path)?;
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS metrics_pr_ages (
    //         date_time TEXT NOT NULL UNIQUE,
    //         p0 INTEGER NOT NULL,
    //         p50 INTEGER NOT NULL,
    //         p100 INTEGER NOT NULL,
    //         mean INTEGER NOT NULL
    //     )",
    //     NO_PARAMS,
    // )?;

    conn.execute(
        "INSERT INTO metrics_pr_ages (date_time, p0, p50, p100, mean) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![datetime, p0, p50, p100, mean],
    )?;

    Ok(())
}

#[derive(Debug)]
pub struct TupleTimeSeries {
    pub points: Vec<(DateTime<Utc>, i64, i64, i64, i64)>,
}

pub fn get_metrics_pull_requests_ages(
    config: &super::super::settings::Database,
) -> Result<TupleTimeSeries> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!(
            "Reading all [pull_requests_ages] metrics into Metrics_Database [{:?}]",
            db_path
        );
    }

    let conn = Connection::open(db_path)?;
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS metrics_pr_ages (
    //         date_time TEXT NOT NULL UNIQUE,
    //         p0 INTEGER NOT NULL,
    //         p50 INTEGER NOT NULL,
    //         p100 INTEGER NOT NULL,
    //         mean INTEGER NOT NULL
    //     )",
    //     NO_PARAMS,
    // )?;

    let mut stmt = conn.prepare("SELECT date_time, p0, p50, p100, mean FROM metrics_pr_ages")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut points = Vec::new();
    while let Some(row) = rows.next()? {
        points.push((
            row.get("date_time")?,
            row.get("p0")?,
            row.get("p50")?,
            row.get("p100")?,
            row.get("mean")?,
        ));
    }

    let timeseries = TupleTimeSeries { points: points };

    Ok(timeseries)
}

pub fn save_metrics_endpoints_num(
    config: &super::super::settings::Database,
    datetime: DateTime<Utc>,
    size: i32,
) -> Result<()> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!(
            "Saving metrics_endpoints_num into Metrics_Database [{:?}]",
            db_path
        );
    }

    let conn = Connection::open(db_path)?;
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS metrics_endpoints_num (
    //         date_time TEXT NOT NULL UNIQUE,
    //         value INTEGER NOT NULL
    //     )",
    //     NO_PARAMS,
    // )?;

    conn.execute(
        "INSERT INTO metrics_endpoints_num (date_time, value) VALUES (?1, ?2)",
        params![datetime, size],
    )?;

    Ok(())
}

pub fn get_metrics_endpoints_number(
    config: &super::super::settings::Database,
) -> Result<TimeSeries> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!(
            "Reading all [metrics_endpoints_num] metrics into Metrics_Database [{:?}]",
            db_path
        );
    }

    let conn = Connection::open(db_path)?;
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS metrics_endpoints_num (
    //         date_time TEXT NOT NULL UNIQUE,
    //         value INTEGER NOT NULL
    //     )",
    //     NO_PARAMS,
    // )?;

    let mut stmt = conn.prepare("SELECT date_time, value FROM metrics_endpoints_num")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut points = Vec::new();
    while let Some(row) = rows.next()? {
        let time = row.get("date_time")?;
        let val = row.get("value")?;

        points.push((time, val));
    }

    let timeseries = TimeSeries { points: points };

    Ok(timeseries)
}

pub fn save_metrics_zally_ignore(
    config: &super::super::settings::Database,
    datetime: DateTime<Utc>,
    stats: std::collections::HashMap<i64, usize>,
) -> Result<()> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!(
            "Saving [metrics_zally_ignore] metrics into Metrics_Database [{:?}]",
            db_path
        );
    }

    let conn = Connection::open(db_path)?;

    let stats_as_yaml = serde_yaml::to_string(&stats)
        .unwrap_or(String::from("Error: Unable to get yaml from stats"));
    debug!("Saving stats {:?}", stats_as_yaml);
    conn.execute(
        "INSERT INTO metrics_zally_ignore (date_time, data_points) VALUES (?1, ?2)",
        params![datetime, stats_as_yaml],
    )?;
    Ok(())
}

pub fn save_metrics_endpoints_num_per_audience(
    config: &super::super::settings::Database,
    datetime: DateTime<Utc>,
    stats: std::collections::HashMap<String, usize>,
) -> Result<()> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!(
            "Saving [metrics_endpoints_per_audience] metrics into Metrics_Database [{:?}]",
            db_path
        );
    }

    let conn = Connection::open(db_path)?;

    let stats_as_yaml = serde_yaml::to_string(&stats)
        .unwrap_or(String::from("Error: Unable to get yaml from stats"));
    debug!("Saving stats {:?}", stats_as_yaml);
    conn.execute(
        "INSERT INTO metrics_endpoints_per_audience (date_time, data_points) VALUES (?1, ?2)",
        params![datetime, stats_as_yaml],
    )?;
    Ok(())
}

#[derive(Debug)]
pub struct i64BasedTimeSeries {
    pub points: Vec<(DateTime<Utc>, std::collections::HashMap<i64, usize>)>,
}

pub fn get_metrics_zally_ignore(
    config: &super::super::settings::Database,
) -> Result<i64BasedTimeSeries> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!(
            "Reading all [metrics_zally_ignore] metrics from Metrics_Database [{:?}]",
            db_path
        );
    }

    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT date_time, data_points FROM metrics_zally_ignore")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut points = Vec::new();
    while let Some(row) = rows.next()? {
        let time = row.get("date_time")?;
        let val: String = row.get("data_points")?;
        points.push((
            time,
            serde_yaml::from_str(val.as_str()).unwrap_or(std::collections::HashMap::new()),
        ));
    }

    let timeseries = i64BasedTimeSeries { points: points };

    Ok(timeseries)
}

#[derive(Debug)]
pub struct StringBasedTimeSeries {
    pub points: Vec<(DateTime<Utc>, std::collections::HashMap<String, usize>)>,
}

pub fn get_metrics_endpoints_per_audience(
    config: &super::super::settings::Database,
) -> Result<StringBasedTimeSeries> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!(
            "Reading all [metrics_endpoints_per_audience] metrics from Metrics_Database [{:?}]",
            db_path
        );
    }

    let conn = Connection::open(db_path)?;
    //TODO - code mutualization w/ get_metrics_zally_ignore
    let mut stmt =
        conn.prepare("SELECT date_time, data_points FROM metrics_endpoints_per_audience")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut points = Vec::new();
    while let Some(row) = rows.next()? {
        let time = row.get("date_time")?;
        let val: String = row.get("data_points")?;
        points.push((
            time,
            serde_yaml::from_str(val.as_str()).unwrap_or(std::collections::HashMap::new()),
        ));
    }

    let timeseries = StringBasedTimeSeries { points: points };

    Ok(timeseries)
}
