use crate::Interpreter;
use swc_ecma_ast::Ident;
use yavashark_env::scope::Scope;
use yavashark_env::{ControlFlow, Realm, RuntimeResult};

impl Interpreter {
    pub fn run_ident(realm: &mut Realm, stmt: &Ident, scope: &mut Scope) -> RuntimeResult {
        let ident = stmt.sym.to_string();
        let value = scope.resolve(&ident, realm)?;
        value.map_or_else(
            || {
                if scope.is_strict_mode()? {
                    Err(ControlFlow::error_reference(format!(
                        "{ident} is not defined"
                    )))
                } else {
                    Ok(yavashark_env::Value::Undefined)
                }
            },
            Ok,
        )
    }
}
