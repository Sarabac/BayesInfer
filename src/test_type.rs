use std::{borrow::Borrow, collections::HashMap, fs, marker::PhantomData, result};

use futures::{
    future::{join_all, try_join_all},
    join, FutureExt, StreamExt, TryFutureExt,
};
use serde::{de::DeserializeOwned, Serialize};
use sqlx::{
    postgres::{PgPoolOptions, PgQueryResult, PgRow},
    query::Query,
    Connection, Executor, PgConnection, PgPool, Pool, Postgres, Row,
};

const INIT_FILE: &str = "src/init.sql";
pub struct BayesModel<H: Serialize + DeserializeOwned, R: Serialize + DeserializeOwned> {
    conn: PgPool,
    hypo: PhantomData<H>,
    record: PhantomData<R>,
}

impl<H: Serialize + DeserializeOwned, R: Serialize + DeserializeOwned> BayesModel<H, R> {
    pub async fn connect(database_url: &str) -> BayesModel<H, R> {
        println!("connect");
        let conn = PgPool::connect(database_url)
            .await
            .expect("can not connect");
        let result = BayesModel {
            conn,
            hypo: PhantomData::<H>,
            record: PhantomData::<R>,
        };
        println!("clear");
        result.clear().await;
        println!("init");
        result.init().await;
        result
    }

    pub async fn init(&self) {
        let init = fs::read_to_string(INIT_FILE).expect("no init file");
        sqlx::query(&init).execute_many(&self.conn).await;
    }

    pub async fn clear(&self) {
        sqlx::query(
            "
        DROP TABLE IF EXISTS model CASCADE;
        DROP TABLE IF EXISTS  likelihood CASCADE;
        DROP TABLE IF EXISTS  hypo CASCADE;
        DROP TABLE IF EXISTS  record CASCADE;
        ",
        )
        .execute_many(&self.conn)
        .await;
    }

    async fn add<T, I>(&self, hypos: I, request: &str) -> Result<u64, sqlx::Error>
    where
        T: Serialize,
        I: IntoIterator,
        I::Item: Borrow<T>,
    {
        let mut nb_write = 0;
        for hypo in hypos {
            let h: &T = hypo.borrow();
            let serial = serde_json::to_string(h).unwrap();
            let result = sqlx::query(request)
                .bind(serial)
                .execute(&self.conn)
                .await?;
            nb_write += result.rows_affected();
        }
        Ok(nb_write)
    }

    pub async fn add_hypo<I>(&self, hypos: I) -> Result<u64, sqlx::Error>
    where
        I: IntoIterator,
        I::Item: Borrow<H>,
    {
        self.add(hypos, r"INSERT INTO hypo(json_serial) VALUES($1)")
            .await
    }

    pub async fn add_record<I>(&self, records: I) -> Result<u64, sqlx::Error>
    where
        I: IntoIterator,
        I::Item: Borrow<R>,
    {
        self.add(records, r"INSERT INTO record(json_serial) VALUES($1)")
            .await
    }

    pub async fn init_prior<F>(&self, func: F) -> usize
    where
        F: Fn(&H) -> f64,
    {
        sqlx::query("SELECT id, json_serial FROM hypo")
            .fetch(&self.conn)
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
                return sqlx::query(
                    "INSERT INTO likelihood(hypo_id, iter, proba) VALUES($1, 0, $2)",
                )
                .bind(r.0)
                .bind(r.1)
                .execute(&self.conn)
                .await
                .ok();
            })
            .count()
            .await
    }

    async fn write_hypo(&self, serial: String, proba: f64) -> Result<(), sqlx::Error> {
        sqlx::query(r"INSERT INTO hypo(json_serial) VALUES($1) RETURNING id")
            .bind(serial)
            .fetch_one(&self.conn)
            .map(|row| {
                let id: i32 = row?.try_get(0)?;
                let rid = Ok::<i32, sqlx::Error>(id);
                rid.map(|id| (id, proba))
            })
            .then(|res| async move {
                let (id, proba) = res?;
                sqlx::query(r"INSERT INTO likelihood(hypo_id, iter, proba) VALUES($1, 0, $2)")
                    .bind(id)
                    .bind(proba)
                    .execute(&self.conn)
                    .await
                    .map(|_| ())
            })
            .await
    }

    pub async fn define_hypo<I>(&self, hypos: I) -> bool
    where
        I: IntoIterator,
        I::Item: Borrow<(H, f64)>,
    {
        let all_threads = hypos
            .into_iter()
            .map(|row| {
                let (hypo, proba): &(H, f64) = row.borrow().clone();
                let serial = serde_json::to_string(hypo).unwrap();
                return (serial, *proba);
            })
            .map(|(serial, proba)| self.write_hypo(serial, proba));

        join_all(all_threads)
            .await
            .into_iter()
            .all(|res| match res {
                Ok(_) => true,
                Err(_) => false,
            })
    }

    async fn get_data<T: DeserializeOwned>(&self, get_query: &str) -> HashMap<i32, T> {
        sqlx::query(get_query)
            .fetch(&self.conn)
            .filter_map(|row| async move {
                let id: i32 = row.as_ref().ok()?.try_get(0).ok()?;
                let json_serial: String = row.as_ref().ok()?.try_get(1).ok()?;
                let data: T = serde_json::from_str(&json_serial).ok()?;
                return Some((id, data));
            })
            .collect()
            .await
    }

    pub async fn get_hypo(&self) -> HashMap<i32, H> {
        self.get_data(r"Select id, json_serial FROM hypo").await
    }

    pub async fn get_record(&self) -> HashMap<i32, R> {
        self.get_data(r"Select id, json_serial FROM record").await
    }

    async fn write_model(
        &self,
        hypo_id: i32,
        record_id: i32,
        proba: f64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO model(hypo_id, record_id, proba) VALUES($1, $2, $3)")
            .bind(hypo_id)
            .bind(record_id)
            .bind(proba)
            .execute(&self.conn)
            .await
            .map(|_| ())
    }

    pub async fn add_model_fn<F>(&self, func: F) -> (i32, i32)
    where
        F: Fn(&H, &R) -> f64,
    {
        sqlx::query(
            "SELECT h.id, r.id, h.json_serial, r.json_serial FROM hypo AS h CROSS JOIN record AS r",
        )
        .fetch(&self.conn)
        .map(|res_row| {
            if let Ok(row) = res_row {
                let h_id: i32 = row.try_get(0).ok()?;
                let r_id: i32 = row.try_get(1).ok()?;
                let h_json: &str = row.try_get(2).ok()?;
                let r_json: &str = row.try_get(3).ok()?;
                let hypo: H = serde_json::from_str(&h_json).ok()?;
                let record: R = serde_json::from_str(&r_json).ok()?;
                let proba: f64 = func(&hypo, &record);
                if proba == 0f64 {
                    return None;
                } else {
                    return Some((h_id, r_id, proba));
                }
            }
            return None;
        })
        .filter_map(|row| async move { row })
        .then(|(hypo_id, record_id, proba)| self.write_model(hypo_id, record_id, proba))
        .fold((0i32, 0i32), |acc, curr| async move {
            match curr {
                Ok(_) => (acc.0 + 1i32, acc.1),
                Err(_) => (acc.0, acc.1 + 1),
            }
        })
        .await
    }
}
