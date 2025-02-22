use swc_common::Spanned;
use crate::Interpreter;
use swc_ecma_ast::SeqExpr;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult, Value};

impl Interpreter {
    pub fn run_seq(realm: &mut Realm, stmt: &SeqExpr, scope: &mut Scope) -> RuntimeResult {
        let mut last = None;
        
        for expr in &stmt.exprs {
            last = Some(Self::run_expr(realm, expr, expr.span(), scope)?);
        }

        Ok(last.unwrap_or(Value::Undefined))
    }
}
