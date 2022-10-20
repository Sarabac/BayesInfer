use std::{borrow::Borrow, fs, result};

use futures::{Future, StreamExt, TryStreamExt};
use postgres::{Client, Error};
use serde::{de::DeserializeOwned, Serialize};
use sqlx::{
    postgres::{PgPoolOptions, PgRow},
    query::Query,
    Connection, Executor, PgConnection, PgPool, Pool, Postgres, Row,
};

pub async fn connect(database_url: &str, init_file: &str) -> Result<PgPool, sqlx::Error> {
    println!("connect");
    let client = PgPool::connect(database_url).await?;
    println!("open init file");
    let init = fs::read_to_string(init_file).expect("no init file");
    clear(&client).await?;
    println!("init table");
    sqlx::query(&init).execute_many(&client).await;
    Ok(client)
}

async fn add<H, I>(client: &PgPool, hypos: I, request: &str) -> Result<u64, sqlx::Error>
where
    H: Serialize,
    I: IntoIterator,
    I::Item: Borrow<H>,
{
    let mut nb_write = 0;
    for hypo in hypos {
        let h: &H = hypo.borrow();
        let serial = serde_json::to_string(h).unwrap();
        let result = sqlx::query(request).bind(serial).execute(client).await?;
        nb_write += result.rows_affected();
    }
    Ok(nb_write)
}

pub async fn add_hypo<H, I>(client: &PgPool, hypos: I) -> Result<u64, sqlx::Error>
where
    H: Serialize,
    I: IntoIterator,
    I::Item: Borrow<H>,
{
    add(client, hypos, r"INSERT INTO hypo(json_serial) VALUES($1)").await
}

pub async fn add_record<H, I>(client: &PgPool, records: I) -> Result<u64, sqlx::Error>
where
    H: Serialize,
    I: IntoIterator,
    I::Item: Borrow<H>,
{
    add(
        client,
        records,
        r"INSERT INTO record(json_serial) VALUES($1)",
    )
    .await
}

pub async fn init_prior<H, F>(client: &PgPool, func: F) -> usize
where
    H: DeserializeOwned,
    F: Fn(&H) -> f64,
{
    sqlx::query("SELECT id, json_serial FROM hypo")
        .fetch(client)
        .map(|row| {
            let r = row.ok()?;
            let id: i32 = r.try_get(0).ok()?;
            let json_serial: &str = r.try_get(1).ok()?;
            let hypo: H = serde_json::from_str(&json_serial).ok()?;
            let proba: f64 = func(&hypo);
            return Some((id, proba));
        })
        .then(|row| async move {
            let r = row?;
            return sqlx::query("INSERT INTO likelihood(hypo_id, iter, proba) VALUES($1, 0, $2)")
                .bind(r.0)
                .bind(r.1)
                .execute(client)
                .await
                .ok();
        })
        .count()
        .await
}
pub async fn build_model<H, R, F>(client: &PgPool, func: F) -> u64
where
    H: DeserializeOwned,
    R: DeserializeOwned,
    F: Fn(&H, &R) -> f64,
{
    sqlx::query(
        "SELECT h.id, r.id, h.json_serial, r.json_serial FROM hypo AS h CROSS JOIN record AS r",
    )
    .fetch(client)
    .map(|res_row| {
        if let Ok(row) = res_row {
            let h_id: i32 = row.try_get(0).ok()?;
            let r_id: i32 = row.try_get(1).ok()?;
            let h_json: &str = row.try_get(2).ok()?;
            let r_json: &str = row.try_get(3).ok()?;
            let hypo: H = serde_json::from_str(&h_json).ok()?;
            let record: R = serde_json::from_str(&r_json).ok()?;
            let proba: f64 = func(&hypo, &record);
            return Some((h_id, r_id, proba));
        }
        return None;
    })
    .filter_map(|res| async move { res })
    .then(|d| {
        sqlx::query("INSERT INTO model(hypo_id, record_id, proba) VALUES($1, $2, $3)")
            .bind(d.0)
            .bind(d.1)
            .bind(d.2)
            .execute(client)
    })
    .filter_map(|res| async move { res.ok() })
    .fold(0u64, |acc, curr| async move { acc + curr.rows_affected() })
    .await
}

pub async fn read_all(client: &PgPool) -> String {
    sqlx::query("SELECT id, json_serial FROM hypo")
        .map(|r: PgRow| {
            let result: String = r.try_get(1).unwrap_or("".to_string());
            result
        })
        .fetch(client)
        .filter_map(|s| async move { s.ok() })
        .collect::<Vec<String>>()
        .await
        .join("\n")
}

pub async fn read_model(client: &PgPool) -> String {
    sqlx::query("SELECT * FROM model")
        .fetch(client)
        .filter_map(|row| async move {
            if let Ok(r) = row {
                let id: i32 = r.try_get(0).ok()?;
                let id_h: i32 = r.try_get(1).ok()?;
                let id_r: i32 = r.try_get(2).ok()?;
                let proba: f64 = r.try_get(3).ok()?;
                return Some(format!("{}[{}, {}]: {}", id, id_h, id_r, proba));
            }
            return None;
        })
        .collect::<Vec<String>>()
        .await
        .join("\n")
}
pub async fn clear(client: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
    DROP TABLE IF EXISTS model CASCADE;
    DROP TABLE IF EXISTS  likelihood CASCADE;
    DROP TABLE IF EXISTS  hypo CASCADE;
    DROP TABLE IF EXISTS  record CASCADE;
    ",
    )
    .execute_many(client)
    .await;
    Ok(())
}
