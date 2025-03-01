use crate::harness::setup_global;
use crate::metadata::NegativePhase;
use crate::utils::parse_file;
use crate::TEST262_DIR;
use std::path::{Path, PathBuf};
use yavashark_env::error::ErrorObj;
use yavashark_env::scope::Scope;
use yavashark_env::{Error, Realm};
use yavashark_interpreter::Interpreter;

pub fn run_file(file: PathBuf) -> Result<String, Error> {
    let (mut realm, mut scope) = setup_global(file.clone())?;

    run_file_in(file, &mut realm, &mut scope)
}

pub fn run_file_in(file: PathBuf, realm: &mut Realm, scope: &mut Scope) -> Result<String, Error> {
    let (stmt, metadata) = parse_file(&file);

    for inc in metadata.includes {
        let path = Path::new(TEST262_DIR).join("harness").join(inc);

        scope.set_path(path.clone())?;

        run_file_in(path, realm, scope)?;
    }

    scope.set_path(file)?;

    let mut res = Interpreter::run_in(&stmt, realm, scope).and_then(|v| v.to_string(realm));


    if let Some(negative) = metadata.negative {
        if negative.phase == NegativePhase::Runtime {
            let e = match res {
                Ok(v) => {
                    return Err(Error::new_error(format!("Expected error but got {:?}", v)));
                }
                Err(e) => e,
            };

            if negative.ty.is_empty() {
                res = Ok("".to_string());
            } else {
                let Some(err) = scope.resolve(&negative.ty, realm)? else {
                    return Err(Error::ty("Error type not found"));
                };

                let e = ErrorObj::error_to_value(e, realm);

                if !e.instance_of(&err, realm)? {
                    return Err(Error::new_error(format!("Expected error of type {:?} but got {:?}", err, e)));
                }

                res = Ok("".to_string());
            }
        }
    }


    res
}
