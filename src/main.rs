use rust_db::database::{add_hypo, add_record, build_model, clear, connect, read_all, read_model, init_prior};
use serde::{Deserialize, Serialize};

use postgres::Error;
use sqlx::{pool, postgres::PgPoolOptions};

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

fn init_binom(bin: &BinomTest) -> f64 {
    bin.p
}

fn model(bin: &BinomTest, genre: &Genre) -> f64 {
    bin.p * (genre.f as f64)
}

//#[async_std::main]
#[tokio::main]
// #[actix_web::main]
async fn main() -> Result<(), sqlx::Error> {
    let client = connect(DATABASE_URL, INIT_FILE).await?;
    println!("clear database");
    print!("create hypo");
    let iter_binom = (1..100).map(|n| BinomTest {
        p: 1f64 / (n as f64),
    });
    println!("create genre");
    let iter_genre = vec![Genre { f: 0 }, Genre { f: 1 }];

    println!("add hypo");
    add_hypo(&client, iter_binom).await?;
    println!("add genre");
    add_record(&client, iter_genre).await?;
    println!("read all");
    println!("{}", read_all(&client).await);
    println!("add model");
    let nb_model = build_model(&client, model).await;
    println!("{}", nb_model);
    println!("read model");
    let string_model = read_model(&client).await;
    println!("{}", string_model);
    println!("init prior");
    init_prior(&client, |h: &BinomTest| h.p).await;


    Ok(())


}
