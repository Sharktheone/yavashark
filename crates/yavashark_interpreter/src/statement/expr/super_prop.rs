use crate::Interpreter;
use swc_ecma_ast::{SuperProp, SuperPropExpr};
use yavashark_env::print::PrettyPrint;
use yavashark_env::scope::Scope;
use yavashark_env::value::{Obj, Value};
use yavashark_env::{Realm, RuntimeResult};

impl Interpreter {
    pub fn run_super_prop(
        realm: &mut Realm,
        stmt: &SuperPropExpr,
        scope: &mut Scope,
    ) -> RuntimeResult {
        let this = scope.this()?;
        let proto = this.prototype(realm)?
            .to_object()?;
        let sup = proto.prototype(realm)?
            .to_object()?;

        match &stmt.prop {
            SuperProp::Ident(i) => {
                let name = i.sym.to_string();

                Ok(sup.resolve_property(name, realm)?.unwrap_or(Value::Undefined))
            }
            SuperProp::Computed(p) => {
                let name = Self::run_expr(realm, &p.expr, stmt.span, scope)?;

                Ok(sup.resolve_property(name, realm)?.unwrap_or(Value::Undefined))
            }
        }
    }
}
