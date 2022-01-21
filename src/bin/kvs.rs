use anyhow::Result;
use clap::{load_yaml, App};

fn main() -> Result<()> {
    let yaml = load_yaml!("../../cli.yaml");
    let mut app = App::from(yaml);
    let matches = App::from(yaml).get_matches();
    let mut db_store = kvs::KvStore::open(std::path::PathBuf::new())?;
    match matches.subcommand() {
        Some(("set", sub_matches)) => {
            let key = sub_matches.value_of("KEY").expect("Required");
            let value = sub_matches.value_of("VALUE").expect("Required");
            db_store.set(key.to_string(), value.to_string())?;
            //            println!("SET: {} -> {}", key, value);
        }
        Some(("get", sub_matches)) => {
            let key = sub_matches.value_of("KEY").expect("Required");
            match db_store.get(key.to_string()) {
                Ok(Some(value)) => println!("{}", value),
                _ => println!("Key not found"),
            };
        }
        Some(("rm", sub_matches)) => {
            let key = sub_matches.value_of("KEY").expect("Required");
            if db_store.remove(key.to_string()).is_err() {
                println!("Key not found");
                std::process::exit(1);
            }
            //            println!("RM: {}", key);
        }
        _ => {
            eprintln!("\x1b[31;1mNo subcommand found. Plese retry with a valid subcommand\x1b[0m");
            app.print_help()?;
            std::process::exit(1);
        }
    }
    Ok(())
}
