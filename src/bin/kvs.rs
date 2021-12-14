use anyhow::Result;
use clap::{load_yaml, App};

fn main() -> Result<()> {
    let yaml = load_yaml!("../../cli.yaml");
    let matches = App::from(yaml).get_matches();
    let mut db_store = kvs::KvStore::new();
    match matches.subcommand() {
        Some(("set", sub_matches)) => {
            let key = sub_matches.value_of("KEY").expect("Required");
            let value = sub_matches.value_of("VALUE").expect("Required");
            db_store.set(key.to_string(), value.to_string())?;
            println!("SET: {} -> {}", key, value);
        }
        Some(("get", sub_matches)) => {
            let key = sub_matches.value_of("KEY").expect("Required");
            println!("GET: {} -> {}", key, db_store.get(key.to_string())?);
        }
        Some(("rm", sub_matches)) => {
            let key = sub_matches.value_of("KEY").expect("Required");
            db_store.remove(key.to_string())?;
            println!("RM: {}", key);
        }
        _ => unreachable!(),
    }
    Ok(())
}
