use crate::error::get_error;
use crate::realm::Realm;
use crate::{get_console, ObjectHandle, Res, Variable};

pub fn init_global_obj(obj: &ObjectHandle, realm: &Realm) -> Res {
    let mut obj = obj.get_mut()?;

    obj.define_variable(
        "console".into(),
        Variable::new_read_only(get_console(realm)),
    );

    obj.define_variable("Error".into(), Variable::new_read_only(get_error(realm)));

    #[allow(clippy::expect_used)]
    obj.define_variable("Array".into(), realm.intrinsics.array_constructor());

    Ok(())
}
