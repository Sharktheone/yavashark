use crate::test262::{print, Test262};
use crate::utils::parse_file;
use crate::{ObjectHandle, TEST262_FALLBACK_DIR};
use std::fs;
use std::path::{Path, PathBuf};
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, Res};
use yavashark_interpreter::eval::InterpreterEval;
use yavashark_interpreter::Interpreter;
use crate::metadata::Flags;

const NON_RAW_HARNESS: [&str; 2] = ["harness/assert.js", "harness/sta.js"];

pub fn run_harness_in_realm(realm: &mut Realm, scope: &mut Scope, p: &Path) -> Res {
    let path = scope.get_current_path()?;

    let compiled = NON_RAW_HARNESS.iter().map(|f| {
        let path = p.join(Path::new(f));

        (parse_file(path.as_path()).0, path) //TODO: if sta.js or assert.js has actually some metadata, this needs to be changed
    });

    for (s, path) in compiled {
        scope.set_path(path.to_path_buf())?;

        Interpreter::run_program_in(&s, realm, scope)?;
    }

    scope.set_path(path)?;

    Ok(())
}

pub fn run_async_in_realm(realm: &mut Realm, scope: &mut Scope, harness: &Path) -> Res {
    let path = scope.get_current_path()?;

    let async_path = harness.join("harness/doneprintHandle.js");

    let (prog, meta) = parse_file(async_path.as_path());
    
    if meta.flags.contains(Flags::ONLY_STRICT) {
        scope.set_strict_mode()?;
    }

    scope.set_path(async_path)?;

    Interpreter::run_program_in(&prog, realm, scope)?;

    scope.set_path(path)?;

    Ok(())
}

pub fn setup_global(file: PathBuf, raw: bool, async_: bool, strict: bool) -> Res<(Realm, Scope, PathBuf)> {
    #[cfg(feature = "timings")]
    let now = std::time::Instant::now();
    let mut r = Realm::new()?;
    #[cfg(feature = "timings")]
    unsafe {
        crate::REALM_DURATION = now.elapsed();
    }
    let mut s = Scope::global(&r, file);
    
    if strict {
        s.set_strict_mode()?;
    }

    let t262 = ObjectHandle::new(Test262::new(&mut r));

    let global = r.global.clone();

    global.define_property("$262".into(), t262.into(), &mut r)?;

    let print = print(&mut r).into();
    global.define_property("print".into(), print, &mut r)?;

    let p = s.get_current_path()?;

    let harness_dir =
        find_test_root_from_test(&p).unwrap_or_else(|| Path::new(TEST262_FALLBACK_DIR));

    if !raw {
        run_harness_in_realm(&mut r, &mut s, &harness_dir)?;

        if async_ {
            run_async_in_realm(&mut r, &mut s, harness_dir)?;
        }
    }

    r.set_eval(InterpreterEval, strict)?;
    yavashark_vm::init(&mut r)?;

    Ok((r, s, harness_dir.to_path_buf()))
}

fn find_test_root_from_test(test_path: &Path) -> Option<&Path> {
    let mut current = test_path;

    while let Some(parent) = current.parent() {
        if parent.ends_with("test") {
            let root = parent.parent()?;

            let harness = root.join("harness");
            let test = root.join("test");

            if fs::metadata(&harness).is_ok() && fs::metadata(&test).is_ok() {
                return Some(root);
            }
        }
        current = parent;
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::harness::setup_global;
    use std::path::PathBuf;

    #[test]
    fn new_harness() {
        let (_global, _scope, _) = match setup_global(PathBuf::new(), false, false, false) {
            Ok(v) => v,
            Err(e) => {
                panic!("Failed to create new harness: {e}")
            }
        };
    }
}
