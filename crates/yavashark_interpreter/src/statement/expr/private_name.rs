use crate::Interpreter;
use swc_ecma_ast::PrivateName;
use yavashark_env::scope::Scope;
use yavashark_env::{Class, ClassInstance, Error, Realm, RuntimeResult};

impl Interpreter {
    pub fn run_private_name(
        realm: &mut Realm,
        stmt: &PrivateName,
        scope: &mut Scope,
    ) -> RuntimeResult {
        let name = stmt.name.as_str();

        let this = scope.this()?;

        let Some(class) = this.downcast::<ClassInstance>()? else {
            return Err(Error::ty_error("Private name can only be used in class".into()).into());
        };

        class
            .get_private_prop(name)?
            .ok_or(Error::ty_error(format!("Private name {name} not found")).into())
    }
}
