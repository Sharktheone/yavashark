mod direct_exec;
#[cfg(target_arch = "wasm32")]
mod wasm;

use std::path::PathBuf;
use swc_common::BytePos;
use swc_common::input::StringInput;
use swc_ecma_parser::{Parser, Syntax};
pub use direct_exec::*;
use yavashark_env::{Error, Realm, ValueResult};
use yavashark_env::scope::Scope;
use yavashark_interpreter::Interpreter;

pub struct Engine {
    realm: Realm,
}

impl Engine {
    pub fn new() -> Self {
        let realm = Realm::new().expect("failed to create realm");
        Self { realm }
    }

    pub fn new_global_scope(&mut self, path: impl Into<PathBuf>) -> Scope {
        Scope::global(&mut self.realm, path.into())
    }

    pub fn realm(&mut self) -> &mut Realm {
        &mut self.realm
    }

    pub fn run_in(&mut self, scope: &mut Scope, script: &str) -> ValueResult {
        let input = StringInput::new(script, BytePos(0), BytePos(script.len() as u32));

        let script = Parser::new(Syntax::Es(Default::default()), input, None)
            .parse_script()
            .map_err(|e| Error::syn_error(format!("{e:?}")))?;


        let result = Interpreter::run_in(&script.body, &mut self.realm, scope);
        result
    }

    pub fn run(&mut self, script: &str, path: &str) -> ValueResult {
        let mut scope = Scope::global(&mut self.realm, PathBuf::from(path));

        self.run_in(&mut scope, script)
    }
}