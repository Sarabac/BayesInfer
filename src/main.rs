use std::fs;

use postgres::{Client, Error, NoTls};

const DATABASE_URL: &str = "postgresql://spatial:pass@localhost:5243/geopython";
const INIT_FILE: &str = "src/init.sql";

fn main() -> Result<(), Error> {
    println!("connect");
    let mut client = Client::connect(DATABASE_URL, NoTls)?;
    println!("open init file");
    let init_file = fs::read_to_string(INIT_FILE).expect("no init file");
    println!("init table");
    client.batch_execute(&init_file)?;

    client.execute("INSERT INTO hypo(json_serial) VALUES($1)", &[&"test"])?;


    let query = client.query("SELECT * FROM hypo", &[])?;
    for row in query {
      let id: i32 = row.get(0);
      let seri: &str = row.get(1);
      println!("voici: {}:{}", id, seri);
        
    }

    Ok(())
    
}