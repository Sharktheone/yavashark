use crate::Interpreter;
use swc_ecma_ast::WithStmt;
use yavashark_env::{Realm, RuntimeResult, Value};

use crate::scope::Scope;

impl Interpreter {
    pub fn run_with(realm: &mut Realm, stmt: &WithStmt, scope: &mut Scope) -> RuntimeResult {
        let obj = Self::run_expr(realm, &stmt.obj, stmt.span, scope)?;

        let mut scope = scope.child()?;

        for (key, value) in obj.properties()? {
            let Value::String(key) = key else {
                continue;
            };

            scope.declare_var(key, value)?;
        }

        Self::run_statement(realm, &stmt.body, &mut scope)
    }
}
