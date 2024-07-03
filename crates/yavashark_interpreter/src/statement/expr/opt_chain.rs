use crate::Interpreter;
use swc_ecma_ast::{OptChainBase, OptChainExpr};
use yavashark_env::scope::Scope;
use yavashark_env::{Context, ControlFlow, RuntimeResult, Value};

impl Interpreter {
    pub fn run_opt_chain(
        ctx: &mut Context,
        stmt: &OptChainExpr,
        scope: &mut Scope,
    ) -> RuntimeResult {
        let is_first_optional = !scope.state_is_opt_chain()?;

        let mut scope_new = if is_first_optional {
            let mut scope = Scope::with_parent(scope)?;
            scope.state_set_opt_chain();
            Some(scope)
        } else {
            None
        };

        let scope = scope_new.as_mut().unwrap_or(scope);

        let res = run(stmt, scope, ctx);

        if res == Err(ControlFlow::OptChainShortCircuit) && is_first_optional {
            Ok(Value::Undefined)
        } else {
            res
        }
    }
}

fn run(stmt: &OptChainExpr, scope: &mut Scope, ctx: &mut Context) -> RuntimeResult {
    match &*stmt.base {
        OptChainBase::Member(member) => {
            let value = Interpreter::run_expr(ctx, &member.obj, member.span, scope)?;

            if (value == Value::Undefined || value == Value::Null) && stmt.optional {
                return Err(ControlFlow::OptChainShortCircuit);
            }

            Interpreter::run_member_on(ctx, value, &member.prop, member.span, scope)
        }
        OptChainBase::Call(call) => {
            let callee = Interpreter::run_expr(ctx, &call.callee, call.span, scope)?;

            println!("{:?} is {}", callee, stmt.optional);

            if (callee == Value::Undefined || callee == Value::Null) && stmt.optional {
                return Err(ControlFlow::OptChainShortCircuit);
            }

            Ok(Interpreter::run_call_on(
                ctx, callee, &call.args, call.span, scope,
            )?)
        }
    }
}
