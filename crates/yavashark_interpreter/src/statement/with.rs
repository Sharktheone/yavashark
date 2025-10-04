use crate::Interpreter;
use swc_ecma_ast::WithStmt;
use yavashark_env::{PropertyKey, Realm, RuntimeResult, Value};

use crate::scope::Scope;

impl Interpreter {
    pub fn run_with(realm: &mut Realm, stmt: &WithStmt, scope: &mut Scope) -> RuntimeResult {
        let obj = Self::run_expr(realm, &stmt.obj, stmt.span, scope)?;

        let mut scope = scope.child()?;

        for (key, value) in obj.properties(realm)? {
            let PropertyKey::String(key) = key else {
                continue;
            };

            scope.declare_var(key.to_string(), value, realm)?;
        }

        Self::run_statement(realm, &stmt.body, &mut scope)
    }
}
