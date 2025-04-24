use crate::builtins::{get_decode_uri, get_decode_uri_component, get_encode_uri, get_encode_uri_component, get_escape, get_is_finite, get_is_nan, get_parse_float, get_parse_int};
use crate::error::get_error;
use crate::realm::Realm;
use crate::Value;
use crate::{get_console, ObjectHandle, Res, Variable};

pub fn init_global_obj(handle: &ObjectHandle, realm: &Realm) -> Res {
    let obj = handle.guard();

    obj.define_variable(
        "undefined".into(),
        Variable::new_read_only(Value::Undefined),
    )?;
    obj.define_variable(
        "NaN".into(),
        Variable::new_read_only(Value::Number(f64::NAN)),
    )?;
    obj.define_variable(
        "Infinity".into(),
        Variable::new_read_only(Value::Number(f64::INFINITY)),
    )?;
    obj.define_variable("null".into(), Variable::new_read_only(Value::Null))?;
    obj.define_variable("true".into(), Variable::new_read_only(Value::Boolean(true)))?;
    obj.define_variable(
        "false".into(),
        Variable::new_read_only(Value::Boolean(false)),
    )?;

    obj.define_variable("console".into(), Variable::write_config(get_console(realm)))?;

    obj.define_variable("Error".into(), Variable::write_config(get_error(realm)?))?;

    #[allow(clippy::expect_used)]
    obj.define_variable(
        "Array".into(),
        Variable::write_config(realm.intrinsics.array_constructor().value),
    )?;

    obj.define_variable(
        "Object".into(),
        Variable::write_config(realm.intrinsics.obj_constructor().value),
    )?;
    obj.define_variable(
        "Function".into(),
        Variable::write_config(realm.intrinsics.func_constructor().value),
    )?;
    obj.define_variable(
        "Math".into(),
        Variable::write_config(realm.intrinsics.math_obj().value),
    )?;
    obj.define_variable(
        "String".into(),
        Variable::write_config(realm.intrinsics.string_constructor().value),
    )?;
    obj.define_variable(
        "Number".into(),
        Variable::write_config(realm.intrinsics.number_constructor().value),
    )?;
    obj.define_variable(
        "Boolean".into(),
        Variable::write_config(realm.intrinsics.boolean_constructor().value),
    )?;
    obj.define_variable(
        "Symbol".into(),
        Variable::write_config(realm.intrinsics.symbol_constructor().value),
    )?;
    obj.define_variable(
        "BigInt".into(),
        Variable::write_config(realm.intrinsics.bigint_constructor().value),
    )?;
    obj.define_variable(
        "RegExp".into(),
        Variable::write_config(realm.intrinsics.regexp_constructor().value),
    )?;
    obj.define_variable(
        "JSON".into(),
        Variable::write_config(realm.intrinsics.json_obj().value),
    )?;
    obj.define_variable(
        "TypeError".into(),
        Variable::write_config(realm.intrinsics.type_error_constructor().value),
    )?;
    obj.define_variable(
        "RangeError".into(),
        Variable::write_config(realm.intrinsics.range_error_constructor().value),
    )?;
    obj.define_variable(
        "ReferenceError".into(),
        Variable::write_config(realm.intrinsics.reference_error_constructor().value),
    )?;
    obj.define_variable(
        "SyntaxError".into(),
        Variable::write_config(realm.intrinsics.syntax_error_constructor().value),
    )?;
    obj.define_variable(
        "EvalError".into(),
        Variable::write_config(realm.intrinsics.eval_error_constructor().value),
    )?;
    obj.define_variable(
        "URIError".into(),
        Variable::write_config(realm.intrinsics.uri_error_constructor().value),
    )?;

    obj.define_variable(
        "globalThis".into(),
        Variable::write_config(realm.global.clone().into()),
    )?;
    obj.define_variable(
        "global".into(),
        Variable::write_config(realm.global.clone().into()),
    )?;
    obj.define_variable(
        "ArrayBuffer".into(),
        Variable::write_config(realm.intrinsics.arraybuffer_constructor().value),
    )?;
    obj.define_variable(
        "DataView".into(),
        Variable::write_config(realm.intrinsics.data_view_constructor().value),
    )?;

    obj.define_variable(
        "Int8Array".into(),
        Variable::write_config(realm.intrinsics.int8array_constructor().value),
    )?;

    obj.define_variable(
        "Uint8Array".into(),
        Variable::write_config(realm.intrinsics.uint8array_constructor().value),
    )?;

    obj.define_variable(
        "Uint8ClampedArray".into(),
        Variable::write_config(realm.intrinsics.uint8clampedarray_constructor().value),
    )?;

    obj.define_variable(
        "Int16Array".into(),
        Variable::write_config(realm.intrinsics.int16array_constructor().value),
    )?;

    obj.define_variable(
        "Uint16Array".into(),
        Variable::write_config(realm.intrinsics.uint16array_constructor().value),
    )?;

    obj.define_variable(
        "Int32Array".into(),
        Variable::write_config(realm.intrinsics.int32array_constructor().value),
    )?;

    obj.define_variable(
        "Uint32Array".into(),
        Variable::write_config(realm.intrinsics.uint32array_constructor().value),
    )?;

    obj.define_variable(
        "Float16Array".into(),
        Variable::write_config(realm.intrinsics.float16array_constructor().value),
    )?;

    obj.define_variable(
        "Float32Array".into(),
        Variable::write_config(realm.intrinsics.float32array_constructor().value),
    )?;

    obj.define_variable(
        "Float64Array".into(),
        Variable::write_config(realm.intrinsics.float64array_constructor().value),
    )?;

    obj.define_variable(
        "BigInt64Array".into(),
        Variable::write_config(realm.intrinsics.bigint64array_constructor().value),
    )?;

    obj.define_variable(
        "BigUint64Array".into(),
        Variable::write_config(realm.intrinsics.biguint64array_constructor().value),
    )?;

    obj.define_variable("escape".into(), Variable::write_config(get_escape(realm)))?;
    obj.define_variable("unescape".into(), Variable::write_config(get_escape(realm)))?;
    obj.define_variable(
        "encodeURI".into(),
        Variable::write_config(get_encode_uri(realm)),
    )?;
    obj.define_variable(
        "decodeURI".into(),
        Variable::write_config(get_decode_uri(realm)),
    )?;
    obj.define_variable(
        "encodeURIComponent".into(),
        Variable::write_config(get_encode_uri_component(realm)),
    )?;
    obj.define_variable(
        "decodeURIComponent".into(),
        Variable::write_config(get_decode_uri_component(realm)),
    )?;
    obj.define_variable(
        "Map".into(),
        Variable::write_config(realm.intrinsics.map_constructor().value),
    )?;
    obj.define_variable(
        "Set".into(),
        Variable::write_config(realm.intrinsics.set_constructor().value),
    )?;
    obj.define_variable(
        "Date".into(),
        Variable::write_config(realm.intrinsics.date_constructor().value),
    )?;
    obj.define_variable(
        "Reflect".into(),
        Variable::write_config(realm.intrinsics.reflect_obj().value),
    )?;
    obj.define_variable(
        "Temporal".into(),
        Variable::write_config(realm.intrinsics.temporal_obj().value),
    )?;

    obj.define_variable(
        "Promise".into(),
        Variable::write_config(realm.intrinsics.promise_constructor().value),
    )?;

    macro_rules! copy_from {
        ($prop:ident, $name:ident) => {
            obj.define_variable(
                stringify!($name).into(),
                Variable::write_config(
                    realm
                        .intrinsics
                        .$prop
                        .resolve_property_no_get_set(&stringify!($name).into())?
                        .map(|x| x.value)
                        .unwrap_or(Value::Undefined),
                ),
            )?;
        };

        (c, $prop:ident, $name:ident) => {
            obj.define_variable(
                stringify!($name).into(),
                Variable::write_config(
                    realm
                        .intrinsics
                        .$prop()
                        .value
                        .as_object()?
                        .resolve_property_no_get_set(&stringify!($name).into())?
                        .map(|x| x.value)
                        .unwrap_or(Value::Undefined),
                ),
            )?;
        };
    }

    obj.define_variable(
        "parseInt".into(),
        Variable::write_config(get_parse_int(realm).into()),
    )?;
    
    obj.define_variable(
        "parseFloat".into(),
        Variable::write_config(get_parse_float(realm).into()),
    )?;

    obj.define_variable(
        "isNaN".into(),
        Variable::write_config(get_is_nan(realm).into()),
    )?;
    obj.define_variable(
        "isFinite".into(),
        Variable::write_config(get_is_finite(realm).into()),
    )?;

    #[cfg(feature = "out-of-spec-experiments")]
    crate::experiments::init(handle, realm)?;

    Ok(())
}
