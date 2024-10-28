use crate::error::get_error;
use crate::realm::Realm;
use crate::{get_console, ObjectHandle, Res, Value, Variable};

pub fn init_global_obj(obj: ObjectHandle, realm: &mut Realm) -> Res {
    let mut obj = obj.get_mut()?;

    obj.define_variable(
        "undefined".into(),
        Variable::new_read_only(Value::Undefined),
    );

    obj.define_variable(
        "NaN".into(),
        Variable::new_read_only(Value::Number(f64::NAN)),
    );

    obj.define_variable(
        "Infinity".into(),
        Variable::new_read_only(Value::Number(f64::INFINITY)),
    );

    obj.define_variable("null".into(), Variable::new_read_only(Value::Null));

    obj.define_variable("true".into(), Variable::new_read_only(Value::Boolean(true)));

    obj.define_variable(
        "false".into(),
        Variable::new_read_only(Value::Boolean(false)),
    );

    obj.define_variable(
        "console".into(),
        Variable::new_read_only(get_console(realm)),
    );

    obj.define_variable("Error".into(), Variable::new_read_only(get_error(realm)));

    #[allow(clippy::expect_used)]
    obj.define_variable(
        "Array".into(),
        realm
            .intrinsics
            .array
            .get_property(&"constructor".into())
            .expect("Failed to get Array constructor") //This can only happen when we have a programming error
            .into(),
    );

    Ok(())
}
