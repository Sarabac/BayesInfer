use std::{borrow::Borrow, fs};

use postgres::{Client, Error, NoTls};
use serde::{de::DeserializeOwned, Serialize};

pub fn connect(database_url: &str, init_file: &str) -> Result<Client, Error> {
    println!("connect");
    let mut client = Client::connect(database_url, NoTls)?;
    println!("open init file");
    let init = fs::read_to_string(init_file).expect("no init file");
    println!("init table");
    clear(&mut client)?;
    client.batch_execute(&init)?;
    Ok(client)
}

fn add<H, I>(client: &mut Client, hypos: I, request: &str) -> Result<u64, Error>
where
    H: Serialize,
    I: IntoIterator,
    I::Item: Borrow<H>,
{
    let mut nb_write = 0;
    for hypo in hypos {
        let h: &H = hypo.borrow();
        let serial = serde_json::to_string(h).unwrap();
        nb_write += client.execute(request, &[&serial])?;
    }
    Ok(nb_write)
}

pub fn add_hypo<H, I>(client: &mut Client, hypos: I) -> Result<u64, Error>
where
    H: Serialize,
    I: IntoIterator,
    I::Item: Borrow<H>,
{
    add(client, hypos, "INSERT INTO hypo(json_serial) VALUES($1)")
}

pub fn add_record<H, I>(client: &mut Client, records: I) -> Result<u64, Error>
where
    H: Serialize,
    I: IntoIterator,
    I::Item: Borrow<H>,
{
    add(
        client,
        records,
        "INSERT INTO record(json_serial) VALUES($1)",
    )
}

pub fn init_prior<H, F>(client: &mut Client, func: F) -> Result<u64, Error>
where
    H: DeserializeOwned,
    F: Fn(&H) -> f64,
{
    let hypos = client.query("SELECT id, json_serial FROM hypo", &[])?;
    let zero: u32 = 0;
    let mut nb: u64 = 0;
    for row in hypos {
        let id: i32 = row.get(0);
        let json_serial: &str = row.get(1);
        let hypo: H = serde_json::from_str(&json_serial).unwrap();
        let proba: f64 = func(&hypo);
        nb += client.execute(
            "INSERT INTO likelihood(hypo_id, iter, proba) VALUES($1, $2, $3)",
            &[&id, &zero, &proba],
        )?;
    }
    Ok(nb)
}

pub fn build_model<H, R, F>(client: &mut Client, func: F) -> Result<u64, Error>
where
    H: DeserializeOwned,
    R: DeserializeOwned,
    F: Fn(&H, &R) -> f64,
{
    let hypos_record = client.query(
        "SELECT h.id, r.id, h.json_serial, r.json_serial FROM hypo AS h CROSS JOIN record AS r",
        &[],
    )?;
    let mut nb: u64 = 0;
    for row in hypos_record {
        let h_id: i32 = row.get(0);
        let r_id: i32 = row.get(1);
        let h_json: &str = row.get(2);
        let r_json: &str = row.get(3);
        let hypo: H = serde_json::from_str(&h_json).unwrap();
        let record: R = serde_json::from_str(&r_json).unwrap();
        let proba: f64 = func(&hypo, &record);
        if proba != 0f64 {
            nb += client.execute(
                "INSERT INTO model(hypo_id, record_id, proba) VALUES($1, $2, $3)",
                &[&h_id, &r_id, &proba],
            )?;
        }
    }
    Ok(nb)
}

pub fn read_all(client: &mut Client) -> Result<u64, Error> {
    let query = client.query("SELECT * FROM hypo", &[])?;
    for row in query {
        let id: i32 = row.get(0);
        let serial: &str = row.get(1);
        println!("{} : {}", id, serial);
    }
    Ok(0u64)
}

pub fn read_model(client: &mut Client) -> Result<u64, Error> {
    let query = client.query("SELECT * FROM model", &[])?;
    for row in query {
        let id: i32 = row.get(0);
        let id_h: i32 = row.get(1);
        let id_r: i32 = row.get(2);
        let proba: f64 = row.get(3);
        println!("{}[{}, {}]: {}", id, id_h, id_r, proba);
    }
    Ok(0u64)
}
pub fn clear(client: &mut Client) -> Result<u64, Error> {
    client.execute("DROP TABLE IF EXISTS model CASCADE", &[])?;
    client.execute("DROP TABLE IF EXISTS  likelihood CASCADE", &[])?;
    client.execute("DROP TABLE IF EXISTS  hypo CASCADE", &[])?;
    client.execute("DROP TABLE IF EXISTS  record CASCADE", &[])?;
    Ok(0u64)
}
