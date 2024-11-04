use crate::test262::{print, Test262};
use crate::{Error, ObjectHandle, TEST262_DIR};
use anyhow::anyhow;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_ecma_ast::Stmt;
use swc_ecma_parser::{EsSyntax, Parser, Syntax};
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, Res, Result};
use yavashark_interpreter::Interpreter;

const NON_RAW_HARNESS: [&str; 2] = ["harness/assert.js", "harness/sta.js"];

static COMPILED: LazyLock<Vec<(Vec<Stmt>, PathBuf)>> = LazyLock::new(|| {
    let p = Path::new(TEST262_DIR);

    NON_RAW_HARNESS
        .iter()
        .map(|f| {
            let path = p.join(Path::new(f));

            (parse_file(path.as_path()), path)
        })
        .collect()
});

fn parse_file(f: &Path) -> Vec<Stmt> {
    let input = std::fs::read_to_string(f).unwrap();

    if input.is_empty() {
        return Vec::new();
    }

    let input = StringInput::new(&input, BytePos(0), BytePos(input.len() as u32 - 1));

    let c = EsSyntax::default();

    let mut p = Parser::new(Syntax::Es(c), input, None);

    p.parse_script().unwrap().body
}

pub fn run_harness_in_realm(realm: &mut Realm, scope: &mut Scope) -> Res {
    let path = scope.get_current_path()?;

    for (s, path) in &*COMPILED {
        scope.set_path(path.to_path_buf())?;

        Interpreter::run_in(s, realm, scope)?;
    }

    scope.set_path(path)?;

    Ok(())
}

pub fn setup_global(file: PathBuf) -> Result<(Realm, Scope)> {
    let mut r = Realm::new()?;
    let mut s = Scope::global(&r, file);

    let t262 = ObjectHandle::new(Test262::new(&r));

    r.global.define_property("$262".into(), t262.into())?;

    let print = print(&mut r).into();
    r.global.define_property("print".into(), print)?;

    run_harness_in_realm(&mut r, &mut s)?;

    Ok((r, s))
}

#[cfg(test)]
mod tests {
    use crate::harness::setup_global;
    use std::path::PathBuf;

    #[test]
    fn new_harness() {
        let (_global, _scope) = match setup_global(PathBuf::new()) {
            Ok(v) => v,
            Err(e) => {
                panic!("Failed to create new harness: {e}")
            }
        };
    }
}
