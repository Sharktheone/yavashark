use swc_ecma_ast::{ArrowExpr, BlockStmtOrExpr};
use yavashark_macro::object;
use yavashark_value::{Error, Func};

use crate::context::Context;
use crate::{ControlFlow, FunctionPrototype, RuntimeResult, ValueResult};
use crate::scope::Scope;
use crate::Value;
use crate::object::Object;
use crate::Variable;


#[object]
#[derive(Debug)]
pub struct ArrowFunction {
    expr: ArrowExpr,
    this: Value,
    scope: Scope,
}


impl Func<Context> for ArrowFunction {
    fn call(&mut self, ctx: &mut Context, args: Vec<Value>, _this: Value) -> ValueResult {
        
        let scope = &mut self.scope.child();
        scope.state_set_function();
        
        for (pat, value) in self.expr.params.iter().zip(args.iter()) {
            ctx.run_pat(pat, scope, value.copy())?;
        }
        
        let res = match &*self.expr.body {
            BlockStmtOrExpr::BlockStmt(stmt) => {
                ctx.run_block(stmt, scope)
            }
            BlockStmtOrExpr::Expr(expr) => {
                ctx.run_expr(expr, self.expr.span, scope)
            }
        };
        
        
        match res {
            Ok(val) => Ok(Value::Undefined),
            Err(ControlFlow::Return(val)) => Ok(val),
            Err(ControlFlow::Error(e)) => Err(e),
            _ => Ok(res?), //will always be Err
        }
    }
}


impl Context {
    pub fn run_arrow(&mut self, stmt: &ArrowExpr, scope: &mut Scope) -> RuntimeResult {
        let this = scope.this.copy();
        
        let arrow = ArrowFunction {
            object: Object::raw_with_proto(self.func_prototype.clone().into()),
            expr: stmt.clone(),
            this,
            scope: scope.clone(),
        };
        
        Ok(arrow.into_func_value())
    }
}
