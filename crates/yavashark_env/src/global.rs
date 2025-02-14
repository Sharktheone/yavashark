use yavashark_value::Value;
use crate::error::get_error;
use crate::realm::Realm;
use crate::{get_console, ObjectHandle, Res, Variable};


pub fn init_global_obj(handle: &ObjectHandle, realm: &Realm) -> Res {
    let obj = handle.get();

    obj.define_variable(
        "console".into(),
        Variable::new_read_only(get_console(realm)),
    )?;

    obj.define_variable("Error".into(), Variable::new_read_only(get_error(realm)))?;

    #[allow(clippy::expect_used)]
    obj.define_variable("Array".into(), realm.intrinsics.array_constructor())?;

    obj.define_variable("Object".into(), realm.intrinsics.obj_constructor())?;
    obj.define_variable("Function".into(), realm.intrinsics.func_constructor())?;
    obj.define_variable("Math".into(), realm.intrinsics.math_obj())?;
    obj.define_variable("String".into(), realm.intrinsics.string_constructor())?;
    obj.define_variable("Number".into(), realm.intrinsics.number_constructor())?;
    obj.define_variable("Boolean".into(), realm.intrinsics.boolean_constructor())?;
    obj.define_variable("Symbol".into(), realm.intrinsics.symbol_constructor())?;
    obj.define_variable("BigInt".into(), realm.intrinsics.bigint_constructor())?;
    obj.define_variable("RegExp".into(), realm.intrinsics.regexp_constructor())?;
    obj.define_variable("JSON".into(), realm.intrinsics.json_obj())?;
    obj.define_variable("TypeError".into(), realm.intrinsics.type_error_constructor())?;
    obj.define_variable("RangeError".into(), realm.intrinsics.range_error_constructor())?;
    obj.define_variable("ReferenceError".into(), realm.intrinsics.reference_error_constructor())?;
    obj.define_variable("SyntaxError".into(), realm.intrinsics.syntax_error_constructor())?;
    
    macro_rules! copy_from {
    ($prop:ident, $name:ident) => {
        obj.define_variable(stringify!($name).into(), realm.intrinsics.$prop.resolve_property_no_get_set(&stringify!($name).into())?.map(|x| x.value).unwrap_or(Value::Undefined).into())?;
    };
}
    
    copy_from!(math, isNaN);
    copy_from!(math, isFinite);
    copy_from!(math, parseInt);
    copy_from!(math, parseFloat);

    #[cfg(feature = "out-of-spec-experiments")]
    crate::experiments::init(handle, realm)?;

    Ok(())
}

