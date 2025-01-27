use crate::Interpreter;
use swc_ecma_ast::ClassExpr;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult};
use yavashark_value::Obj;
use crate::class::{create_class, decl_class};

impl Interpreter {
    pub fn run_class(realm: &mut Realm, stmt: &ClassExpr, scope: &mut Scope) -> RuntimeResult {
        let (class, statics) = create_class(realm, &stmt.class, scope, String::new())?;

        let this = class.into_value();

        for static_block in statics {
            Self::run_block_this(realm, &static_block, scope, this.copy())?;
        }
        
        Ok(this)
    }
}
