use std::path::PathBuf;
use yavashark_env::print::PrettyPrint;
use yavashark_test262::run::run_file;

fn main() {
    let mut args = std::env::args();

    args.next();

    let f = args.next().expect("please provide a test path");

    let path = PathBuf::from(f);

    match run_file(path) {
        Err(e) => println!("FAIL:\n {}", e.pretty_print()),
        Ok(v) => println!("PASS:\n {v}"),
    }
}
