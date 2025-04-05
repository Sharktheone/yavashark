use crate::test262::{print, Test262};
use crate::utils::parse_file;
use crate::{ObjectHandle, TEST262_DIR};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use swc_ecma_ast::Program;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, Res};
use yavashark_interpreter::eval::InterpreterEval;
use yavashark_interpreter::Interpreter;

const NON_RAW_HARNESS: [&str; 2] = ["harness/assert.js", "harness/sta.js"];

static COMPILED: LazyLock<Vec<(Program, PathBuf)>> = LazyLock::new(|| {
    let p = Path::new(TEST262_DIR);

    NON_RAW_HARNESS
        .iter()
        .map(|f| {
            let path = p.join(Path::new(f));

            (parse_file(path.as_path()).0, path) //TODO: if sta.js or assert.js has actually some metadata, this needs to be changed
        })
        .collect()
});

pub fn run_harness_in_realm(realm: &mut Realm, scope: &mut Scope) -> Res {
    let path = scope.get_current_path()?;

    for (s, path) in &*COMPILED {
        scope.set_path(path.to_path_buf())?;

        Interpreter::run_program_in(s, realm, scope)?;
    }

    scope.set_path(path)?;

    Ok(())
}

pub fn run_async_in_realm(realm: &mut Realm, scope: &mut Scope) -> Res {
    let path = scope.get_current_path()?;

    let async_path = Path::new(TEST262_DIR).join("harness/doneprintHandle.js");
    
    let prog = parse_file(async_path.as_path()).0;

    scope.set_path(async_path)?;

    Interpreter::run_program_in(&prog, realm, scope)?;

    scope.set_path(path)?;

    Ok(())
}

pub fn setup_global(file: PathBuf, raw: bool, async_: bool) -> Res<(Realm, Scope)> {
    let mut r = Realm::new()?;
    let mut s = Scope::global(&r, file);

    let t262 = ObjectHandle::new(Test262::new(&r));

    r.global.define_property("$262".into(), t262.into())?;

    let print = print(&mut r).into();
    r.global.define_property("print".into(), print)?;

    if !raw {
        run_harness_in_realm(&mut r, &mut s)?;
        
        if async_ {
            run_async_in_realm(&mut r, &mut s)?;
        }
    }
    

    r.set_eval(InterpreterEval)?;

    Ok((r, s))
}

#[cfg(test)]
mod tests {
    use crate::harness::setup_global;
    use std::path::PathBuf;

    #[test]
    fn new_harness() {
        let (_global, _scope) = match setup_global(PathBuf::new(), false, false) {
            Ok(v) => v,
            Err(e) => {
                panic!("Failed to create new harness: {e}")
            }
        };
    }
}
