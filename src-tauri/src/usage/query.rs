use std::collections::BTreeMap;

use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, TimeZone, Timelike};
use rusqlite::types::Value;
use rusqlite::{params_from_iter, Connection};
use serde::{Deserialize, Serialize};

use super::store::Db;
use crate::error::Result;

const MAX_BUCKETS: usize = 5000;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Bucket {
    Hour,
    Day,
    Week,
    Month,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Filters {
    pub profile_id: Option<String>,
    pub mode: Option<String>,
    pub app: Option<String>,
    pub domain: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportArgs {
    pub from: i64,
    pub to: i64,
    pub bucket: Bucket,
    #[serde(default)]
    pub filters: Filters,
    #[serde(default)]
    pub compare: bool,
}

#[derive(Clone, Copy, Default, Serialize)]
pub struct Point {
    pub start: i64,
    pub up: u64,
    pub down: u64,
}

#[derive(Clone, Copy, Default, Serialize)]
pub struct Totals {
    pub up: u64,
    pub down: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub series: Vec<Point>,
    pub total: Totals,
    pub previous: Option<Totals>,
    pub peak: Option<Point>,
    pub connected_secs: u64,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TopKind {
    App,
    Domain,
    Profile,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopArgs {
    pub kind: TopKind,
    pub from: i64,
    pub to: i64,
    pub limit: u32,
    #[serde(default)]
    pub filters: Filters,
}

#[derive(Serialize)]
pub struct TopRow {
    pub key: String,
    pub up: u64,
    pub down: u64,
}

pub fn local_time(ts: i64) -> DateTime<Local> {
    Local.timestamp_opt(ts, 0).single().unwrap_or_else(|| Local.timestamp_opt(0, 0).unwrap())
}

pub fn midnight(date: NaiveDate) -> i64 {
    let at = |hour| date.and_hms_opt(hour, 0, 0).and_then(|t| Local.from_local_datetime(&t).earliest());
    at(0).or_else(|| at(1)).map(|d| d.timestamp()).unwrap_or(0)
}

pub fn bucket_start(ts: i64, bucket: Bucket) -> i64 {
    let time = local_time(ts);
    let date = time.date_naive();
    match bucket {
        Bucket::Hour => ts - i64::from(time.minute() * 60 + time.second()),
        Bucket::Day => midnight(date),
        Bucket::Week => midnight(date - Duration::days(i64::from(date.weekday().num_days_from_monday()))),
        Bucket::Month => midnight(date.with_day(1).unwrap_or(date)),
    }
}

pub fn next_bucket(start: i64, bucket: Bucket) -> i64 {
    let date = local_time(start).date_naive();
    match bucket {
        Bucket::Hour => start + 3600,
        Bucket::Day => midnight(date + Duration::days(1)),
        Bucket::Week => midnight(date + Duration::days(7)),
        Bucket::Month => {
            let (year, month) = if date.month() == 12 { (date.year() + 1, 1) } else { (date.year(), date.month() + 1) };
            midnight(NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(date))
        }
    }
}

type Rows = Vec<(i64, u64, u64)>;

fn run(conn: &Connection, sql: &str, params: &[Value]) -> Result<Rows> {
    let mut statement = conn.prepare(sql)?;
    let rows = statement.query_map(params_from_iter(params.iter()), |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?.max(0) as u64, row.get::<_, i64>(2)?.max(0) as u64))
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

fn rows(conn: &Connection, from: i64, to: i64, filters: &Filters) -> Result<(Rows, u64)> {
    let dimension = filters
        .app
        .clone()
        .map(|key| ("app", key))
        .or_else(|| filters.domain.clone().map(|key| ("domain", key)));

    if let Some((kind, key)) = dimension {
        let mut sql = String::from(
            "SELECT ts_hour, up, down FROM usage_dim WHERE kind = ?1 AND key = ?2 AND ts_hour >= ?3 AND ts_hour < ?4",
        );
        let mut params: Vec<Value> = vec![kind.to_string().into(), key.into(), from.into(), to.into()];
        if let Some(profile) = &filters.profile_id {
            sql.push_str(" AND profile_id = ?5");
            params.push(profile.clone().into());
        }
        return Ok((run(conn, &sql, &params)?, 0));
    }

    let (condition, params) = base_condition(from, to, filters);
    let sql = format!(
        "SELECT ts, up, down FROM usage_minute WHERE {condition} UNION ALL SELECT ts, up, down FROM usage_hour WHERE {condition}"
    );
    let seconds: i64 = conn.query_row(
        &format!(
            "SELECT (SELECT COUNT(*) * 60 FROM usage_minute WHERE {condition}) + (SELECT COALESCE(SUM(secs), 0) FROM usage_hour WHERE {condition})"
        ),
        params_from_iter(params.iter()),
        |row| row.get(0),
    )?;
    Ok((run(conn, &sql, &params)?, seconds.max(0) as u64))
}

fn base_condition(from: i64, to: i64, filters: &Filters) -> (String, Vec<Value>) {
    let mut condition = String::from("ts >= ?1 AND ts < ?2");
    let mut params: Vec<Value> = vec![from.into(), to.into()];
    if let Some(profile) = &filters.profile_id {
        params.push(profile.clone().into());
        condition.push_str(&format!(" AND profile_id = ?{}", params.len()));
    }
    if let Some(mode) = &filters.mode {
        params.push(mode.clone().into());
        condition.push_str(&format!(" AND mode = ?{}", params.len()));
    }
    (condition, params)
}

fn totals(rows: &Rows) -> Totals {
    rows.iter().fold(Totals::default(), |t, (_, up, down)| Totals { up: t.up + up, down: t.down + down })
}

pub fn series(rows: &Rows, from: i64, to: i64, bucket: Bucket) -> Vec<Point> {
    let mut sums: BTreeMap<i64, (u64, u64)> = BTreeMap::new();
    for (ts, up, down) in rows {
        let entry = sums.entry(bucket_start(*ts, bucket)).or_default();
        entry.0 += up;
        entry.1 += down;
    }
    let mut points = Vec::new();
    let mut start = bucket_start(from, bucket);
    while start < to && points.len() < MAX_BUCKETS {
        let (up, down) = sums.get(&start).copied().unwrap_or_default();
        points.push(Point { start, up, down });
        start = next_bucket(start, bucket);
    }
    points
}

pub fn report(db: &Db, args: &ReportArgs) -> Result<Report> {
    let (rows, connected_secs) = rows(db.conn(), args.from, args.to, &args.filters)?;
    let series = series(&rows, args.from, args.to, args.bucket);
    let total = totals(&rows);
    let peak = series.iter().filter(|p| p.up + p.down > 0).max_by_key(|p| p.up + p.down).copied();
    let previous = if args.compare {
        let length = args.to - args.from;
        Some(totals(&rows_only(db.conn(), args.from - length, args.from, &args.filters)?))
    } else {
        None
    };
    Ok(Report { series, total, previous, peak, connected_secs })
}

fn rows_only(conn: &Connection, from: i64, to: i64, filters: &Filters) -> Result<Rows> {
    Ok(rows(conn, from, to, filters)?.0)
}

pub fn top(db: &Db, args: &TopArgs) -> Result<Vec<TopRow>> {
    let conn = db.conn();
    let limit = i64::from(args.limit.clamp(1, 200));
    let (sql, params): (String, Vec<Value>) = match args.kind {
        TopKind::App | TopKind::Domain => {
            let kind = if matches!(args.kind, TopKind::App) { "app" } else { "domain" };
            let mut sql = String::from(
                "SELECT key, SUM(up), SUM(down) FROM usage_dim WHERE kind = ?1 AND ts_hour >= ?2 AND ts_hour < ?3",
            );
            let mut params: Vec<Value> = vec![kind.to_string().into(), args.from.into(), args.to.into()];
            if let Some(profile) = &args.filters.profile_id {
                sql.push_str(" AND profile_id = ?4");
                params.push(profile.clone().into());
            }
            sql.push_str(&format!(" GROUP BY key ORDER BY SUM(up) + SUM(down) DESC LIMIT {limit}"));
            (sql, params)
        }
        TopKind::Profile => {
            let (condition, params) = base_condition(args.from, args.to, &args.filters);
            let sql = format!(
                "SELECT profile_id, SUM(up), SUM(down) FROM (
                    SELECT profile_id, up, down FROM usage_minute WHERE {condition}
                    UNION ALL SELECT profile_id, up, down FROM usage_hour WHERE {condition})
                 GROUP BY profile_id ORDER BY SUM(up) + SUM(down) DESC LIMIT {limit}"
            );
            (sql, params)
        }
    };
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(params.iter()), |row| {
        Ok(TopRow {
            key: row.get(0)?,
            up: row.get::<_, i64>(1)?.max(0) as u64,
            down: row.get::<_, i64>(2)?.max(0) as u64,
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn export_csv(db: &Db, args: &ReportArgs) -> Result<String> {
    let report = report(db, args)?;
    let format = match args.bucket {
        Bucket::Hour => "%Y-%m-%d %H:00",
        Bucket::Day => "%Y-%m-%d",
        Bucket::Week => "%Y-%m-%d",
        Bucket::Month => "%Y-%m",
    };
    let mut csv = String::from("start,upload_bytes,download_bytes,total_bytes\n");
    for point in report.series {
        csv.push_str(&format!(
            "{},{},{},{}\n",
            local_time(point.start).format(format),
            point.up,
            point.down,
            point.up + point.down
        ));
    }
    Ok(csv)
}
