use crate::harness::setup_global;
use crate::utils::parse_file;
use crate::TEST262_DIR;
use std::path::{Path, PathBuf};
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, ValueResult};
use yavashark_interpreter::Interpreter;

pub fn run_file(file: PathBuf) -> ValueResult {
    let (mut realm, mut scope) = setup_global(file.clone())?;

    run_file_in(file, &mut realm, &mut scope)
}

pub fn run_file_in(file: PathBuf, realm: &mut Realm, scope: &mut Scope) -> ValueResult {
    let (stmt, metadata) = parse_file(&file);

    for inc in metadata.includes {
        let path = Path::new(TEST262_DIR).join("harness").join(inc);

        scope.set_path(path.clone())?;

        run_file_in(path, realm, scope)?;
    }

    scope.set_path(file)?;

    Interpreter::run_in(&stmt, realm, scope)
}
