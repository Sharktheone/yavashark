use crate::class::{create_class, decl_class};
use crate::Interpreter;
use swc_ecma_ast::ClassExpr;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult};
use yavashark_value::Obj;

impl Interpreter {
    pub fn run_class(realm: &mut Realm, stmt: &ClassExpr, scope: &mut Scope) -> RuntimeResult {
        let name = stmt
            .ident
            .as_ref()
            .map(|id| id.sym.to_string())
            .unwrap_or_default();

        let (class, statics) = create_class(realm, &stmt.class, scope, name)?;

        let this = class.into_value();

        for static_block in statics {
            Self::run_block_this(realm, &static_block, scope, this.copy())?;
        }

        let proto = this.get_property(&"prototype".into(), realm)?;

        proto.define_property("constructor".into(), this.copy());

        Ok(this)
    }
}
