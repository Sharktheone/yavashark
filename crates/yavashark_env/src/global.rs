use crate::error::get_error;
use crate::realm::Realm;
use crate::{get_console, ObjectHandle, Res, Variable};
use yavashark_value::Value;
use crate::builtins::{get_encode_uri, get_encode_uri_component, get_escape};

pub fn init_global_obj(handle: &ObjectHandle, realm: &Realm) -> Res {
    let obj = handle.get();

    obj.define_variable(
        "console".into(),
        Variable::new_read_only(get_console(realm)),
    )?;

    obj.define_variable("Error".into(), Variable::new_read_only(get_error(realm)?))?;

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
    obj.define_variable(
        "TypeError".into(),
        realm.intrinsics.type_error_constructor(),
    )?;
    obj.define_variable(
        "RangeError".into(),
        realm.intrinsics.range_error_constructor(),
    )?;
    obj.define_variable(
        "ReferenceError".into(),
        realm.intrinsics.reference_error_constructor(),
    )?;
    obj.define_variable(
        "SyntaxError".into(),
        realm.intrinsics.syntax_error_constructor(),
    )?;
    obj.define_variable(
        "EvalError".into(),
        realm.intrinsics.eval_error_constructor(),
    )?;
    obj.define_variable("URIError".into(), realm.intrinsics.uri_error_constructor())?;

    obj.define_variable("globalThis".into(), realm.global.clone().into())?;
    obj.define_variable("global".into(), realm.global.clone().into())?;
    obj.define_variable(
        "ArrayBuffer".into(),
        realm.intrinsics.arraybuffer_constructor(),
    )?;
    obj.define_variable("DataView".into(), realm.intrinsics.data_view_constructor())?;
    obj.define_variable("escape".into(), get_escape(realm).into())?;
    obj.define_variable("unescape".into(), get_escape(realm).into())?;
    obj.define_variable("encodeURI".into(), get_encode_uri(realm).into())?;
    obj.define_variable("decodeURI".into(), get_encode_uri(realm).into())?;
    obj.define_variable("encodeURIComponent".into(), get_encode_uri_component(realm).into())?;
    obj.define_variable("decodeURIComponent".into(), get_encode_uri_component(realm).into())?;
    
    

    macro_rules! copy_from {
        ($prop:ident, $name:ident) => {
            obj.define_variable(
                stringify!($name).into(),
                realm
                    .intrinsics
                    .$prop
                    .resolve_property_no_get_set(&stringify!($name).into())?
                    .map(|x| x.value)
                    .unwrap_or(Value::Undefined)
                    .into(),
            )?;
        };

        (c, $prop:ident, $name:ident) => {
            obj.define_variable(
                stringify!($name).into(),
                realm
                    .intrinsics
                    .$prop()
                    .value
                    .as_object()?
                    .resolve_property_no_get_set(&stringify!($name).into())?
                    .map(|x| x.value)
                    .unwrap_or(Value::Undefined)
                    .into(),
            )?;
        };
    }

    copy_from!(c, number_constructor, isNaN);
    copy_from!(c, number_constructor, isFinite);
    copy_from!(c, number_constructor, parseInt);
    copy_from!(c, number_constructor, parseFloat);

    #[cfg(feature = "out-of-spec-experiments")]
    crate::experiments::init(handle, realm)?;

    Ok(())
}
