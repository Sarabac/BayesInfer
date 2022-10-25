use rust_db::bayes_model::BayesModel;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

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

#[sqlx::test(migrations = false)]
async fn birth(conn: PgPool) -> sqlx::Result<()> {
    
    let iter_binom = (1..10).map(|n| BinomTest {
        p: 1f64 / (n as f64),
    });
    
    let iter_genre = vec![Genre { f: 0 }, Genre { f: 1 }];

    
    let model: BayesModel<BinomTest, Genre> = BayesModel::connect(conn).await;

    println!("add hypo");
    model.add_hypo(iter_binom).await?;
    model.add_record(iter_genre).await?;

    let test_get_hypo = model.get_hypo().await;
    assert_eq!(test_get_hypo.len(), 9);
    let nb_model = model.add_model_fn(model_binom).await;
    assert_eq!(nb_model.0, 9 * 2);
    let vec_model = model.get_model_all().await;
    assert_eq!(vec_model.len(), 9 * 2);

    //let string_model = read_model(&client).await;
    //println!("{}", string_model);
    println!("init prior");
    //init_prior(&client, |h: &BinomTest| h.p).await;

    Ok(())
}
