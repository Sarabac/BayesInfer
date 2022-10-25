use rust_db::{
    bayes_model::BayesModel,
};
use serde::{Deserialize, Serialize};

use sqlx::PgPool;

const DATABASE_URL: &str = "postgresql://spatial:pass@localhost:5243/geopython";

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

fn model_binom(bin: &BinomTest, genre: &Genre) -> f64 {
    bin.p * (genre.f as f64)
}

//#[async_std::main]
#[tokio::main]
// #[actix_web::main]
async fn main() -> Result<(), sqlx::Error> {
    let conn = PgPool::connect(DATABASE_URL)
            .await
            .expect("can not connect");


    print!("create hypo");
    let iter_binom = (1..100).map(|n| BinomTest {
        p: 1f64 / (n as f64),
    });
    println!("create genre");
    let iter_genre = vec![Genre { f: 0 }, Genre { f: 1 }];

    println!("create model");
    let model: BayesModel<BinomTest, Genre> = BayesModel::connect(conn).await;

    println!("add hypo");
    model.add_hypo(iter_binom).await?;
    println!("add genre");
    model.add_record(iter_genre).await?;
    println!("read all");
    println!("{:?}", model.get_hypo().await);
    println!("add model");
    let nb_model = model.add_model_fn(model_binom).await;
    println!("{:?}", nb_model);
    println!("read model");
    let vec_model = model.get_model_all().await;
    println!("{:?}", vec_model);
    //let string_model = read_model(&client).await;
    //println!("{}", string_model);
    println!("init prior"); 
    //init_prior(&client, |h: &BinomTest| h.p).await;

    Ok(())
}
