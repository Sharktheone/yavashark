use crate::Interpreter;
use swc_ecma_ast::{SuperProp, SuperPropExpr};
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult};

impl Interpreter {
    pub fn run_super_prop(
        realm: &mut Realm,
        stmt: &SuperPropExpr,
        scope: &mut Scope,
    ) -> RuntimeResult {
        let this = scope.this()?;

        let obj = this.as_object()?;

        let proto = obj.prototype()?;
        let sup = proto.resolve(this, realm)?;
        
        match &stmt.prop {
            SuperProp::Ident(i) => {
                let name = i.sym.to_string();

                Ok(sup.get_property(&name.into(), realm)?)
            }
            SuperProp::Computed(p) => {
                let name = Self::run_expr(realm, &p.expr, stmt.span, scope)?;

                Ok(sup.get_property(&name, realm)?)
            }
        }
    }
}
