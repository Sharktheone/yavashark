use std::cell::{Cell, RefCell};
use yavashark_macro::inline_props;
use crate::builtins::{
    get_decode_uri, get_decode_uri_component, get_encode_uri, get_encode_uri_component, get_escape,
    get_is_finite, get_is_nan, get_parse_float, get_parse_int,
};
use crate::realm::Realm;
use crate::{Object, Value};
use crate::{get_console, ObjectHandle, Res};
use crate::inline_props::InlineObject;
use crate::value::Obj;


#[inline_props(enumerable = false, configurable)]
#[derive(Debug)]
pub struct GlobalProperties {
    #[readonly]
    #[no_configurable]
    undefined: (),

    #[prop("NaN")]
    #[readonly]
    #[no_configurable]
    nan: f64,

    #[prop("Infinity")]
    #[readonly]
    #[no_configurable]
    infinity: f64,

    #[readonly]
    #[no_configurable]
    null: Value,


    #[prop("true")]
    #[readonly]
    #[no_configurable]
    true_: bool,

    #[prop("false")]
    #[readonly]
    #[no_configurable]
    false_: bool,

    console: ObjectHandle,

    #[prop("Error")]
    error: ObjectHandle,

    #[prop("Array")]
    array: ObjectHandle,

    #[prop("Object")]
    object: ObjectHandle,

    #[prop("Function")]
    function: ObjectHandle,

    #[prop("Math")]
    math: ObjectHandle,

    #[prop("String")]
    string: ObjectHandle,

    #[prop("Number")]
    number: ObjectHandle,

    #[prop("Boolean")]
    boolean: ObjectHandle,

    #[prop("Symbol")]
    symbol: ObjectHandle,

    #[prop("BigInt")]
    bigint: ObjectHandle,

    #[prop("RegExp")]
    regexp: ObjectHandle,

    #[prop("JSON")]
    json: ObjectHandle,

    #[prop("TypeError")]
    type_error: ObjectHandle,

    #[prop("RangeError")]
    range_error: ObjectHandle,

    #[prop("ReferenceError")]
    reference_error: ObjectHandle,

    #[prop("SyntaxError")]
    syntax_error: ObjectHandle,

    #[prop("EvalError")]
    eval_error: ObjectHandle,

    #[prop("URIError")]
    uri_error: ObjectHandle,

    #[prop("AggregateError")]
    aggregate_error: ObjectHandle,

    #[prop("globalThis")]
    global_this: ObjectHandle,

    global: ObjectHandle,

    #[prop("ArrayBuffer")]
    array_buffer: ObjectHandle,

    #[prop("SharedArrayBuffer")]
    shared_array_buffer: ObjectHandle,

    #[prop("DataView")]
    data_view: ObjectHandle,

    #[prop("Int8Array")]
    int8_array: ObjectHandle,

    #[prop("Uint8Array")]
    uint8_array: ObjectHandle,

    #[prop("Uint8ClampedArray")]
    uint8_clamped_array: ObjectHandle,

    #[prop("Int16Array")]
    int16_array: ObjectHandle,

    #[prop("Uint16Array")]
    uint16_array: ObjectHandle,

    #[prop("Int32Array")]
    int32_array: ObjectHandle,

    #[prop("Uint32Array")]
    uint32_array: ObjectHandle,

    #[prop("Float16Array")]
    float16_array: ObjectHandle,

    #[prop("Float32Array")]
    float32_array: ObjectHandle,

    #[prop("Float64Array")]
    float64_array: ObjectHandle,

    #[prop("BigInt64Array")]
    bigint64_array: ObjectHandle,

    #[prop("BigUint64Array")]
    biguint64_array: ObjectHandle,

    #[prop("Atomics")]
    atomics: ObjectHandle,

    escape: ObjectHandle,

    unescape: ObjectHandle,

    #[prop("encodeURI")]
    encode_uri: ObjectHandle,

    #[prop("decodeURI")]
    decode_uri: ObjectHandle,

    #[prop("encodeURIComponent")]
    encode_uri_component: ObjectHandle,

    #[prop("decodeURIComponent")]
    decode_uri_component: ObjectHandle,

    #[prop("Map")]
    map: ObjectHandle,

    #[prop("WeakMap")]
    weak_map: ObjectHandle,

    #[prop("WeakRef")]
    weak_ref: ObjectHandle,

    #[prop("Set")]
    set: ObjectHandle,

    #[prop("WeakSet")]
    weak_set: ObjectHandle,

    #[prop("Date")]
    date: ObjectHandle,

    #[prop("Reflect")]
    reflect: ObjectHandle,

    #[prop("Proxy")]
    proxy: ObjectHandle,

    #[prop("Temporal")]
    temporal: ObjectHandle,

    #[prop("Signal")]
    signal: ObjectHandle,

    #[prop("Promise")]
    promise: ObjectHandle,

    #[prop("parseInt")]
    parse_int: ObjectHandle,

    #[prop("parseFloat")]
    parse_float: ObjectHandle,

    #[prop("isNaN")]
    is_nan: ObjectHandle,

    #[prop("isFinite")]
    is_finite: ObjectHandle,

    #[prop("Intl")]
    intl: ObjectHandle,
}


pub fn init_global_obj(realm: &mut Realm) -> Res {
    let inline = GlobalProperties {
        undefined: (),
        nan: f64::NAN,
        infinity: f64::INFINITY,
        null: Value::Null,
        true_: true,
        false_: false,
        console: RefCell::new(get_console(realm)),
        error: RefCell::new(realm.intrinsics.error_constructor()),
        array: RefCell::new(realm.intrinsics.array_constructor()),
        object: RefCell::new(realm.intrinsics.obj_constructor()),
        function: RefCell::new(realm.intrinsics.func_constructor()),
        math: RefCell::new(realm.intrinsics.math_obj()),
        string: RefCell::new(realm.intrinsics.string_constructor()),
        number: RefCell::new(realm.intrinsics.number_constructor()),
        boolean: RefCell::new(realm.intrinsics.boolean_constructor()),
        symbol: RefCell::new(realm.intrinsics.symbol_constructor()),
        bigint: RefCell::new(realm.intrinsics.bigint_constructor()),
        regexp: RefCell::new(realm.intrinsics.regexp_constructor()),
        json: RefCell::new(realm.intrinsics.json_obj()),
        type_error: RefCell::new(realm.intrinsics.type_error_constructor()),
        range_error: RefCell::new(realm.intrinsics.range_error_constructor()),
        reference_error: RefCell::new(realm.intrinsics.reference_error_constructor()),
        syntax_error: RefCell::new(realm.intrinsics.syntax_error_constructor()),
        eval_error: RefCell::new(realm.intrinsics.eval_error_constructor()),
        uri_error: RefCell::new(realm.intrinsics.uri_error_constructor()),
        aggregate_error: RefCell::new(realm.intrinsics.aggregate_error_constructor()),
        global_this: RefCell::new(Object::null()),
        global: RefCell::new(Object::null()),
        array_buffer: RefCell::new(realm.intrinsics.arraybuffer_constructor()),
        shared_array_buffer: RefCell::new(realm.intrinsics.sharedarraybuffer_constructor()),
        data_view: RefCell::new(realm.intrinsics.data_view_constructor()),
        int8_array: RefCell::new(realm.intrinsics.int8array_constructor()),
        uint8_array: RefCell::new(realm.intrinsics.uint8array_constructor()),
        uint8_clamped_array: RefCell::new(realm.intrinsics.uint8clampedarray_constructor()),
        int16_array: RefCell::new(realm.intrinsics.int16array_constructor()),
        uint16_array: RefCell::new(realm.intrinsics.uint16array_constructor()),
        int32_array: RefCell::new(realm.intrinsics.int32array_constructor()),
        uint32_array: RefCell::new(realm.intrinsics.uint32array_constructor()),
        float16_array: RefCell::new(realm.intrinsics.float16array_constructor()),
        float32_array: RefCell::new(realm.intrinsics.float32array_constructor()),
        float64_array: RefCell::new(realm.intrinsics.float64array_constructor()),
        bigint64_array: RefCell::new(realm.intrinsics.bigint64array_constructor()),
        biguint64_array: RefCell::new(realm.intrinsics.biguint64array_constructor()),
        atomics: RefCell::new(realm.intrinsics.atomics_constructor()),
        escape: RefCell::new(get_escape(realm)),
        unescape: RefCell::new(get_escape(realm)),
        encode_uri: RefCell::new(get_encode_uri(realm)),
        decode_uri: RefCell::new(get_decode_uri(realm)),
        encode_uri_component: RefCell::new(get_encode_uri_component(realm)),
        decode_uri_component: RefCell::new(get_decode_uri_component(realm)),
        map: RefCell::new(realm.intrinsics.map_constructor()),
        weak_map: RefCell::new(realm.intrinsics.weak_map_constructor()),
        weak_ref: RefCell::new(realm.intrinsics.weak_ref_constructor()),
        set: RefCell::new(realm.intrinsics.set_constructor()),
        weak_set: RefCell::new(realm.intrinsics.weak_set_constructor()),
        date: RefCell::new(realm.intrinsics.date_constructor()),
        reflect: RefCell::new(realm.intrinsics.reflect_obj()),
        proxy: RefCell::new(realm.intrinsics.proxy_constructor()),
        temporal: RefCell::new(realm.intrinsics.temporal_obj()),
        signal: RefCell::new(realm.intrinsics.signal_obj()),
        promise: RefCell::new(realm.intrinsics.promise_constructor()),
        parse_int: RefCell::new(get_parse_int(realm)),
        parse_float: RefCell::new(get_parse_float(realm)),
        is_nan: RefCell::new(get_is_nan(realm)),
        is_finite: RefCell::new(get_is_finite(realm)),
        intl: RefCell::new(realm.intrinsics.intl_obj()),

        __deleted_properties: Cell::default(),
        __written_properties: Cell::default(),
    };

    let handle = InlineObject::new(inline, realm)
        .into_object();

    {
        #[allow(clippy::unwrap_used)]
        let global = handle.downcast::<InlineObject<GlobalProperties>>().expect("Global object must be InlineObject (unreachable)");

        global.props.global_this.replace(handle.clone());
        global.props.global.replace(handle.clone());

    }


    #[cfg(feature = "out-of-spec-experiments")]
    crate::experiments::init(&handle, realm)?;

    realm.global = handle;

    Ok(())
}
