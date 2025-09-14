use std::path::PathBuf;
use yavashark_env::print::PrettyPrint;
#[cfg(feature = "parser_test")]
use yavashark_test262::parsers::test_file;
#[cfg(not(feature = "parser_test"))]
use yavashark_test262::run::run_file;

const TEST262_ROOT: &str = "../../test262";

fn main() {
    let mut args = std::env::args();

    args.next();

    let f = args.next().expect("please provide a test path");

    let path = if f.starts_with("test/") {
        PathBuf::from(TEST262_ROOT).join(f)
    } else {
        PathBuf::from(f)
    };

    #[cfg(not(feature = "parser_test"))]
    match run_file(path) {
        Err(e) => println!("FAIL:\n {}", e.pretty_print()),
        Ok(v) => println!("PASS:\n {v}"),
    }

    #[cfg(feature = "parser_test")]
    match test_file(path) {
        Err(e) => println!("FAIL:\n {}", e.pretty_print()),
        Ok(v) => println!("PASS:\n {v}"),
    }
}
