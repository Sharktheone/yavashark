use swc_ecma_ast::{ArrowExpr, BlockStmtOrExpr};

use yavashark_env::scope::Scope;
use yavashark_env::value::Func;
use yavashark_env::{
    Context, ControlFlow, Object, ObjectHandle, RuntimeResult, Value, ValueResult,
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

impl Func<Context> for ArrowFunction {
    fn call(&mut self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        let scope = &mut self.scope.child()?;
        scope.state_set_function()?;

        for (pat, value) in self.expr.params.iter().zip(args.iter()) {
            Interpreter::run_pat(ctx, pat, scope, value.copy())?;
        }

        let res = match &*self.expr.body {
            BlockStmtOrExpr::BlockStmt(stmt) => Interpreter::run_block(ctx, stmt, scope),
            BlockStmtOrExpr::Expr(expr) => Interpreter::run_expr(ctx, expr, self.expr.span, scope),
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
            object: Object::raw_with_proto(ctx.proto.func.clone().into()),
            expr: stmt.clone(),
            this,
            scope: scope.clone(),
        };

        let arrow = ObjectHandle::new(arrow);

        Ok(arrow.into())
    }
}
