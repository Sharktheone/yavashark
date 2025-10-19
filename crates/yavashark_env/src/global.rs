use std::cell::{Cell, RefCell};
use yavashark_macro::inline_props;
use crate::builtins::{get_decode_uri, get_decode_uri_component, get_encode_uri, get_encode_uri_component, get_escape, get_is_finite, get_is_nan, get_parse_float, get_parse_int, Atomics, BigIntObj, BooleanObj, Date, Map, NumberObj, Promise, Proxy, RegExp, Set, StringObj, SymbolObj, WeakMap, WeakRef, WeakSet};
use crate::realm::{Intrinsic, Realm};
use crate::{Object, Value};
use crate::{get_console, ObjectHandle, Res};
use crate::array::Array;
use crate::builtins::bigint64array::BigInt64Array;
use crate::builtins::biguint64array::BigUint64Array;
use crate::builtins::buf::ArrayBuffer;
use crate::builtins::dataview::DataView;
use crate::builtins::float16array::Float16Array;
use crate::builtins::float32array::Float32Array;
use crate::builtins::float64array::Float64Array;
use crate::builtins::int16array::Int16Array;
use crate::builtins::int32array::Int32Array;
use crate::builtins::int8array::Int8Array;
use crate::builtins::shared_buf::SharedArrayBuffer;
use crate::builtins::uint16array::Uint16Array;
use crate::builtins::uint32array::Uint32Array;
use crate::builtins::uint8clampedarray::Uint8ClampedArray;
use crate::builtins::unit8array::Uint8Array;
use crate::error_obj::ErrorObj;
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
        error: RefCell::new(ErrorObj::get_global(realm)?),
        array: RefCell::new(Array::get_global(realm)?),
        object: RefCell::new(realm.intrinsics.obj_constructor()),
        function: RefCell::new(realm.intrinsics.func_constructor()),
        math: RefCell::new(realm.intrinsics.math_obj()),
        string: RefCell::new(StringObj::get_global(realm)?),
        number: RefCell::new(NumberObj::get_global(realm)?),
        boolean: RefCell::new(BooleanObj::get_global(realm)?),
        symbol: RefCell::new(SymbolObj::get_global(realm)?),
        bigint: RefCell::new(BigIntObj::get_global(realm)?),
        regexp: RefCell::new(RegExp::get_global(realm)?),
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
        array_buffer: RefCell::new(ArrayBuffer::get_global(realm)?),
        shared_array_buffer: RefCell::new(SharedArrayBuffer::get_global(realm)?),
        data_view: RefCell::new(DataView::get_global(realm)?),
        int8_array: RefCell::new(Int8Array::get_global(realm)?),
        uint8_array: RefCell::new(Uint8Array::get_global(realm)?),
        uint8_clamped_array: RefCell::new(Uint8ClampedArray::get_global(realm)?),
        int16_array: RefCell::new(Int16Array::get_global(realm)?),
        uint16_array: RefCell::new(Uint16Array::get_global(realm)?),
        int32_array: RefCell::new(Int32Array::get_global(realm)?),
        uint32_array: RefCell::new(Uint32Array::get_global(realm)?),
        float16_array: RefCell::new(Float16Array::get_global(realm)?),
        float32_array: RefCell::new(Float32Array::get_global(realm)?),
        float64_array: RefCell::new(Float64Array::get_global(realm)?),
        bigint64_array: RefCell::new(BigInt64Array::get_global(realm)?),
        biguint64_array: RefCell::new(BigUint64Array::get_global(realm)?),
        atomics: RefCell::new(Atomics::get_global(realm)?),
        escape: RefCell::new(get_escape(realm)),
        unescape: RefCell::new(get_escape(realm)),
        encode_uri: RefCell::new(get_encode_uri(realm)),
        decode_uri: RefCell::new(get_decode_uri(realm)),
        encode_uri_component: RefCell::new(get_encode_uri_component(realm)),
        decode_uri_component: RefCell::new(get_decode_uri_component(realm)),
        map: RefCell::new(Map::get_global(realm)?),
        weak_map: RefCell::new(WeakMap::get_global(realm)?),
        weak_ref: RefCell::new(WeakRef::get_global(realm)?),
        set: RefCell::new(Set::get_global(realm)?),
        weak_set: RefCell::new(WeakSet::get_global(realm)?),
        date: RefCell::new(Date::get_global(realm)?),
        reflect: RefCell::new(realm.intrinsics.reflect_obj()),
        proxy: RefCell::new(Proxy::get_global(realm)?),
        temporal: RefCell::new(realm.intrinsics.temporal_obj()),
        signal: RefCell::new(realm.intrinsics.signal_obj()),
        promise: RefCell::new(Promise::get_global(realm)?),
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
