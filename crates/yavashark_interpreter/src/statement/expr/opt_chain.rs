use crate::Interpreter;
use swc_ecma_ast::{OptChainBase, OptChainExpr};
use yavashark_env::scope::Scope;
use yavashark_env::{Context, RuntimeResult, Value};

impl Interpreter {
    pub fn run_opt_chain(
        ctx: &mut Context,
        stmt: &OptChainExpr,
        scope: &mut Scope,
    ) -> RuntimeResult {
        match &*stmt.base {
            OptChainBase::Member(member) => {
                let value = Self::run_expr(ctx, &member.obj, member.span, scope)?;
                
                if (value == Value::Undefined || value == Value::Null) && stmt.optional {
                    return Ok(Value::Undefined);
                }

                Self::run_member_on(ctx, value, &member.prop, member.span, scope)
            }
            OptChainBase::Call(call) => {
                let callee = Self::run_expr(ctx, &call.callee, call.span, scope)?;
                
                if (callee == Value::Undefined || callee == Value::Null) && stmt.optional {
                    return Ok(Value::Undefined);
                }
                
                Ok(Self::run_call_on(ctx, callee, call.args.clone(), call.span, scope, format!("{:?}", call.callee))?)
                
            }
        }
    }
}
