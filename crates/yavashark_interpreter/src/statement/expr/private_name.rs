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

        if let Some(instance) = this.downcast::<ClassInstance>()? {
            let member = instance
                .get_private_prop(name)?
                .ok_or_else(|| Error::ty_error(format!("Private name {name} not found")))?;

            return Self::resolve_private_member(realm, member, this.copy())
                .map(|(value, _)| value);
        }

        if let Some(class) = this.downcast::<Class>()? {
            let member = class
                .get_private_prop(name)
                .ok_or_else(|| Error::ty_error(format!("Private name {name} not found")))?;

            return Self::resolve_private_member(realm, member, this.copy())
                .map(|(value, _)| value);
        }

        Err(Error::ty_error("Private name can only be used in class".into()).into())
    }
}
