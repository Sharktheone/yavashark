use crate::Interpreter;
use std::cmp::max;
use swc_common::Spanned;
use swc_ecma_ast::Tpl;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult};

impl Interpreter {
    pub fn run_tpl(realm: &mut Realm, stmt: &Tpl, scope: &mut Scope) -> RuntimeResult {
        let mut result = String::new();

        let len = max(stmt.quasis.len(), stmt.exprs.len());

        for i in 0..len {
            if let Some(quasi) = stmt.quasis.get(i) {
                result.push_str(quasi.raw.as_ref());
            }

            if let Some(expr) = stmt.exprs.get(i) {
                let value = Self::run_expr(realm, expr, expr.span(), scope)?;
                result.push_str(&value.to_string(realm)?.as_str_lossy());
            }
        }

        Ok(result.into())
    }
}
