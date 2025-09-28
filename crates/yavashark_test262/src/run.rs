use crate::harness::setup_global;
use crate::metadata::{Flags, Metadata, NegativePhase};
use crate::utils::parse_file;
use std::path::{Path, PathBuf};
use swc_ecma_ast::Program;
use yavashark_env::error::ErrorObj;
use yavashark_env::scope::Scope;
use yavashark_env::{Error, Realm};
use yavashark_interpreter::Interpreter;

pub fn run_file(file: PathBuf) -> Result<String, Error> {
    #[cfg(feature = "timings")]
    let parse = std::time::Instant::now();
    let (stmt, metadata) = parse_file(&file);
    #[cfg(feature = "timings")]
    unsafe {
        crate::PARSE_DURATION = parse.elapsed();
    }
    let raw = metadata.flags.contains(Flags::RAW);
    let async_ = metadata.flags.contains(Flags::ASYNC);
    #[cfg(feature = "timings")]
    let setup = std::time::Instant::now();
    let (mut realm, mut scope, harness_dir) = setup_global(file.clone(), raw, async_)?;
    #[cfg(feature = "timings")]
    unsafe {
        crate::SETUP_DURATION = setup.elapsed();
    }

    run_file_in(file, &mut realm, &mut scope, stmt, metadata, &harness_dir)
}

pub fn run_file_in(
    file: PathBuf,
    realm: &mut Realm,
    scope: &mut Scope,
    prog: Program,
    metadata: Metadata,
    harness: &Path,
) -> Result<String, Error> {
    for inc in metadata.includes {
        let path = harness.join("harness").join(inc);

        scope.set_path(path.clone())?;
        let (stmt, metadata) = parse_file(&path);

        run_file_in(path, realm, scope, stmt, metadata, harness)?;
    }

    scope.set_path(file)?;

    let mut res = Interpreter::run_program_in(&prog, realm, scope)
        .and_then(|v| Ok(v.to_string(realm)?.to_string()));

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
                let Some(err) = scope.resolve(&negative.ty)? else {
                    return Err(Error::ty("Error type not found"));
                };

                let e = ErrorObj::error_to_value(e, realm);

                if !e.instance_of(&err, realm)? {
                    return Err(Error::new_error(format!(
                        "Expected error of type {:?} but got {:?}",
                        err, e
                    )));
                }

                res = Ok("".to_string());
            }
        }
    }

    if res.is_ok() {
        tokio::runtime::Builder::new_current_thread()
            .build()?
            .block_on(realm.run_event_loop())
    }

    res
}
