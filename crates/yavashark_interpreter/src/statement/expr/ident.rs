use swc_ecma_ast::Ident;
use yavashark_env::{Context, ControlFlow, RuntimeResult};
use yavashark_env::scope::Scope;
use crate::Interpreter;


impl Interpreter{
    pub fn run_ident(ctx: &mut Context, stmt: &Ident, scope: &mut Scope) -> RuntimeResult {
        let ident = stmt.sym.to_string();
        let value = scope.resolve(&ident)?;
        value.map_or_else(
            || {
                Err(ControlFlow::error_reference(format!(
                    "{ident} is not defined"
                )))
            },
            Ok,
        )
    }
}
