use std::path::{Path, PathBuf};
use yavashark_env::ValueResult;
use yavashark_interpreter::Interpreter;
use crate::harness::setup_global;
use crate::TEST262_DIR;
use crate::utils::parse_file;

pub fn run_file(file: PathBuf) -> ValueResult {
    let path = Path::new(TEST262_DIR).join("test").join(&file);
    let (mut realm, mut scope) = setup_global(file.clone())?;
    
    
    let stmt = parse_file(&path);

    println!("Running...");
    
    Interpreter::run_in(&stmt, &mut realm, &mut scope)
}