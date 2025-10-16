use crate::Interpreter;
use swc_ecma_ast::{OptChainBase, OptChainExpr};
use yavashark_env::scope::Scope;
use yavashark_env::{ControlFlow, Realm, RuntimeResult, Value};

impl Interpreter {
    pub fn run_opt_chain(
        realm: &mut Realm,
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

        let res = run(stmt, scope, realm);

        if res == Err(ControlFlow::OptChainShortCircuit) && is_first_optional {
            Ok(Value::Undefined)
        } else {
            res
        }
    }
}

fn run(stmt: &OptChainExpr, scope: &mut Scope, realm: &mut Realm) -> RuntimeResult {
    match &*stmt.base {
        OptChainBase::Member(member) => {
            let value = Interpreter::run_expr(realm, &member.obj, member.span, scope)?;

            if (value == Value::Undefined || value == Value::Null) && stmt.optional {
                return Err(ControlFlow::OptChainShortCircuit);
            }

            Interpreter::run_member_on(realm, value, &member.prop, member.span, scope)
        }
        OptChainBase::Call(call) => {
            let (callee, this) = Interpreter::run_call_expr(realm, &call.callee, call.span, scope)?;

            if (callee == Value::Undefined || callee == Value::Null) && stmt.optional {
                return Err(ControlFlow::OptChainShortCircuit);
            }

            let this = this.unwrap_or(scope.fn_this()?);

            Ok(Interpreter::run_call_on(
                realm, &callee, this, &call.args, call.span, scope,
            )?)
        }
    }
}
