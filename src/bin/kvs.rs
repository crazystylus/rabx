use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("../../cli.yaml");
    let matches = App::from(yaml).get_matches();
    match matches.subcommand() {
        Some(("set", _)) => panic!("unimplemented"),
        Some(("get", _)) => panic!("unimplemented"),
        Some(("rm", _)) => panic!("unimplemented"),
        _ => unreachable!(),
    }
}
