use crate::builtins::{
    get_decode_uri, get_decode_uri_component, get_encode_uri, get_encode_uri_component, get_escape,
    get_is_finite, get_is_nan, get_parse_float, get_parse_int,
};
use crate::realm::Realm;
use crate::Value;
use crate::{get_console, ObjectHandle, Res, Variable};

pub fn init_global_obj(handle: &ObjectHandle, realm: &mut Realm) -> Res {
    let obj = handle.guard();

    obj.define_property_attributes(
        "undefined".into(),
        Variable::new_read_only(Value::Undefined),
        realm,
    )?;
    obj.define_property_attributes(
        "NaN".into(),
        Variable::new_read_only(Value::Number(f64::NAN)),
        realm,
    )?;
    obj.define_property_attributes(
        "Infinity".into(),
        Variable::new_read_only(Value::Number(f64::INFINITY)),
        realm,
    )?;
    obj.define_property_attributes("null".into(), Variable::new_read_only(Value::Null), realm)?;
    obj.define_property_attributes("true".into(), Variable::new_read_only(Value::Boolean(true)), realm)?;
    obj.define_property_attributes(
        "false".into(),
        Variable::new_read_only(Value::Boolean(false)),
        realm,
    )?;

    obj.define_property_attributes("console".into(), Variable::write_config(get_console(realm)), realm)?;

    obj.define_property_attributes(
        "Error".into(),
        Variable::write_config(realm.intrinsics.error_constructor().value),
        realm,
    )?;

    #[allow(clippy::expect_used)]
    obj.define_property_attributes(
        "Array".into(),
        Variable::write_config(realm.intrinsics.array_constructor().value),
        realm,
    )?;

    obj.define_property_attributes(
        "Object".into(),
        Variable::write_config(realm.intrinsics.obj_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "Function".into(),
        Variable::write_config(realm.intrinsics.func_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "Math".into(),
        Variable::write_config(realm.intrinsics.math_obj().value),
        realm,
    )?;
    obj.define_property_attributes(
        "String".into(),
        Variable::write_config(realm.intrinsics.string_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "Number".into(),
        Variable::write_config(realm.intrinsics.number_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "Boolean".into(),
        Variable::write_config(realm.intrinsics.boolean_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "Symbol".into(),
        Variable::write_config(realm.intrinsics.symbol_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "BigInt".into(),
        Variable::write_config(realm.intrinsics.bigint_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "RegExp".into(),
        Variable::write_config(realm.intrinsics.regexp_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "JSON".into(),
        Variable::write_config(realm.intrinsics.json_obj().value),
        realm,
    )?;
    obj.define_property_attributes(
        "TypeError".into(),
        Variable::write_config(realm.intrinsics.type_error_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "RangeError".into(),
        Variable::write_config(realm.intrinsics.range_error_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "ReferenceError".into(),
        Variable::write_config(realm.intrinsics.reference_error_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "SyntaxError".into(),
        Variable::write_config(realm.intrinsics.syntax_error_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "EvalError".into(),
        Variable::write_config(realm.intrinsics.eval_error_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "URIError".into(),
        Variable::write_config(realm.intrinsics.uri_error_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "AggregateError".into(),
        Variable::write_config(realm.intrinsics.aggregate_error_constructor().value),
        realm,
    )?;

    obj.define_property_attributes(
        "globalThis".into(),
        Variable::write_config(realm.global.clone().into()),
        realm,
    )?;
    obj.define_property_attributes(
        "global".into(),
        Variable::write_config(realm.global.clone().into()),
        realm,
    )?;
    obj.define_property_attributes(
        "ArrayBuffer".into(),
        Variable::write_config(realm.intrinsics.arraybuffer_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "SharedArrayBuffer".into(),
        Variable::write_config(realm.intrinsics.sharedarraybuffer_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "DataView".into(),
        Variable::write_config(realm.intrinsics.data_view_constructor().value),
        realm,
    )?;

    obj.define_property_attributes(
        "Int8Array".into(),
        Variable::write_config(realm.intrinsics.int8array_constructor().value),
        realm,
    )?;

    obj.define_property_attributes(
        "Uint8Array".into(),
        Variable::write_config(realm.intrinsics.uint8array_constructor().value),
        realm,
    )?;

    obj.define_property_attributes(
        "Uint8ClampedArray".into(),
        Variable::write_config(realm.intrinsics.uint8clampedarray_constructor().value),
        realm,
    )?;

    obj.define_property_attributes(
        "Int16Array".into(),
        Variable::write_config(realm.intrinsics.int16array_constructor().value),
        realm,
    )?;

    obj.define_property_attributes(
        "Uint16Array".into(),
        Variable::write_config(realm.intrinsics.uint16array_constructor().value),
        realm,
    )?;

    obj.define_property_attributes(
        "Int32Array".into(),
        Variable::write_config(realm.intrinsics.int32array_constructor().value),
        realm,
    )?;

    obj.define_property_attributes(
        "Uint32Array".into(),
        Variable::write_config(realm.intrinsics.uint32array_constructor().value),
        realm,
    )?;

    obj.define_property_attributes(
        "Float16Array".into(),
        Variable::write_config(realm.intrinsics.float16array_constructor().value),
        realm,
    )?;

    obj.define_property_attributes(
        "Float32Array".into(),
        Variable::write_config(realm.intrinsics.float32array_constructor().value),
        realm,
    )?;

    obj.define_property_attributes(
        "Float64Array".into(),
        Variable::write_config(realm.intrinsics.float64array_constructor().value),
        realm,
    )?;

    obj.define_property_attributes(
        "BigInt64Array".into(),
        Variable::write_config(realm.intrinsics.bigint64array_constructor().value),
        realm,
    )?;

    obj.define_property_attributes(
        "BigUint64Array".into(),
        Variable::write_config(realm.intrinsics.biguint64array_constructor().value),
        realm,
    )?;

    obj.define_property_attributes(
        "Atomics".into(),
        Variable::write_config(realm.intrinsics.atomics_constructor().value),
        realm,
    )?;

    obj.define_property_attributes("escape".into(), Variable::write_config(get_escape(realm)), realm)?;
    obj.define_property_attributes("unescape".into(), Variable::write_config(get_escape(realm)), realm)?;
    obj.define_property_attributes(
        "encodeURI".into(),
        Variable::write_config(get_encode_uri(realm)),
        realm,
    )?;
    obj.define_property_attributes(
        "decodeURI".into(),
        Variable::write_config(get_decode_uri(realm)),
        realm,
    )?;
    obj.define_property_attributes(
        "encodeURIComponent".into(),
        Variable::write_config(get_encode_uri_component(realm)),
        realm,
    )?;
    obj.define_property_attributes(
        "decodeURIComponent".into(),
        Variable::write_config(get_decode_uri_component(realm)),
        realm,
    )?;
    obj.define_property_attributes(
        "Map".into(),
        Variable::write_config(realm.intrinsics.map_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "WeakMap".into(),
        Variable::write_config(realm.intrinsics.weak_map_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "WeakRef".into(),
        Variable::write_config(realm.intrinsics.weak_ref_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "Set".into(),
        Variable::write_config(realm.intrinsics.set_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "WeakSet".into(),
        Variable::write_config(realm.intrinsics.weak_set_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "Date".into(),
        Variable::write_config(realm.intrinsics.date_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "Reflect".into(),
        Variable::write_config(realm.intrinsics.reflect_obj().value),
        realm,
    )?;
    obj.define_property_attributes(
        "Proxy".into(),
        Variable::write_config(realm.intrinsics.proxy_constructor().value),
        realm,
    )?;
    obj.define_property_attributes(
        "Temporal".into(),
        Variable::write_config(realm.intrinsics.temporal_obj().value),
        realm,
    )?;

    obj.define_property_attributes(
        "Signal".into(),
        Variable::write_config(realm.intrinsics.signal_obj().value),
        realm,
    )?;

    obj.define_property_attributes(
        "Promise".into(),
        Variable::write_config(realm.intrinsics.promise_constructor().value),
        realm,
    )?;

    obj.define_property_attributes(
        "parseInt".into(),
        Variable::write_config(get_parse_int(realm).into()),
        realm,
    )?;

    obj.define_property_attributes(
        "parseFloat".into(),
        Variable::write_config(get_parse_float(realm).into()),
        realm,
    )?;

    obj.define_property_attributes(
        "isNaN".into(),
        Variable::write_config(get_is_nan(realm).into()),
        realm,
    )?;
    obj.define_property_attributes(
        "isFinite".into(),
        Variable::write_config(get_is_finite(realm).into()),
        realm,
    )?;

    #[cfg(feature = "out-of-spec-experiments")]
    crate::experiments::init(handle, realm)?;

    Ok(())
}
