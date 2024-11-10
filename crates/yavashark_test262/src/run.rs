use crate::harness::setup_global;
use crate::utils::parse_file;
use crate::TEST262_DIR;
use std::path::{Path, PathBuf};
use yavashark_env::ValueResult;
use yavashark_interpreter::Interpreter;

pub fn run_file(file: PathBuf) -> ValueResult {
    let path = Path::new(TEST262_DIR).join("test").join(&file);
    let (mut realm, mut scope) = setup_global(file.clone())?;

    let stmt = parse_file(&file);

    Interpreter::run_in(&stmt, &mut realm, &mut scope)
}
