use rust_db::database::{clear, connect, add_hypo, add_record, read_all, read_model, build_model};
use serde::{Deserialize, Serialize};

use postgres::Error;

const DATABASE_URL: &str = "postgresql://spatial:pass@localhost:5243/geopython";
const INIT_FILE: &str = "src/init.sql";

#[derive(Serialize, Deserialize, Debug)]
struct BinomTest {
    p: f64,
}
#[derive(Serialize, Deserialize, Debug)]
struct Genre {
    f: i8,
}

fn init_binom(bin: &BinomTest) -> f64{
  bin.p
}

fn model(bin: &BinomTest, genre: &Genre) -> f64 {
  bin.p * (genre.f as f64)
}

fn main() -> Result<(), Error> {
    let mut client = connect(DATABASE_URL, INIT_FILE)?;
    println!("clear database");
    
    print!("create hypo");
    let iter_binom = (1..100).map(|n| BinomTest {
        p: 1f64 / (n as f64),
    });
    println!("create genre");
    let iter_genre = vec![Genre{f:0}, Genre{f:1}];

    println!("add hypo");
    add_hypo(&mut client, iter_binom)?;
    println!("add genre");
    add_record(&mut client, iter_genre)?;
    println!("read all");
    read_all(&mut client)?;

    println!("add model");
    build_model(&mut client, model)?;
    println!("read model");
    read_model(&mut client)?;

    Ok(())
}
