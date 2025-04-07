use std::cell::RefCell;
use std::env::args;
use swc_ecma_ast::{ArrowExpr, BlockStmtOrExpr};

use yavashark_env::scope::Scope;
use yavashark_env::value::Func;
use yavashark_env::{
    ControlFlow, MutObject, Object, ObjectHandle, Realm, RuntimeResult, Value, ValueResult,
};
use yavashark_macro::object;

use crate::Interpreter;

#[object(function)]
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct ArrowFunction {
    expr: ArrowExpr,
    #[gc]
    this: Value,
    #[gc(untyped)]
    scope: Scope,
}

impl Func<Realm> for ArrowFunction {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        let scope = &mut self.scope.child()?;
        scope.state_set_function()?;
        
        let mut args_iter = args.into_iter();

        for pat in self.expr.params.iter() {
            Interpreter::run_pat(realm, pat, scope, &mut args_iter)?;
        }

        let res = match &*self.expr.body {
            BlockStmtOrExpr::BlockStmt(stmt) => Interpreter::run_block(realm, stmt, scope),
            BlockStmtOrExpr::Expr(expr) => {
                match Interpreter::run_expr(realm, expr, self.expr.span, scope) {
                    Ok(value) => return Ok(value),
                    other => other,
                }
            }
        };

        match res {
            Ok(val) => Ok(Value::Undefined),
            Err(ControlFlow::Return(val)) => Ok(val),
            Err(ControlFlow::Error(e)) => Err(e),
            _ => Ok(res?), //res will always be Err, so this will never actually return Ok()
        }
    }
}

impl Interpreter {
    pub fn run_arrow(realm: &mut Realm, stmt: &ArrowExpr, scope: &mut Scope) -> RuntimeResult {
        let this = scope.this()?.copy();

        let arrow = ArrowFunction {
            inner: RefCell::new(MutableArrowFunction {
                object: MutObject::with_proto(realm.intrinsics.func.clone().into()),
            }),
            expr: stmt.clone(),
            this,
            scope: scope.clone(),
        };

        let arrow = ObjectHandle::new(arrow);

        Ok(arrow.into())
    }
}
