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
use crate::partial_init::{Initializer, Partial};
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
    error: Partial<ObjectHandle, GlobalInitializer<ErrorObj>>,

    #[prop("Array")]
    array: Partial<ObjectHandle, GlobalInitializer<Array>>,

    #[prop("Object")]
    object: ObjectHandle,

    #[prop("Function")]
    function: ObjectHandle,

    #[prop("Math")]
    math: ObjectHandle,

    #[prop("String")]
    string: Partial<ObjectHandle, GlobalInitializer<StringObj>>,

    #[prop("Number")]
    number: Partial<ObjectHandle, GlobalInitializer<NumberObj>>,

    #[prop("Boolean")]
    boolean: Partial<ObjectHandle, GlobalInitializer<BooleanObj>>,

    #[prop("Symbol")]
    symbol: Partial<ObjectHandle, GlobalInitializer<SymbolObj>>,

    #[prop("BigInt")]
    bigint: Partial<ObjectHandle, GlobalInitializer<BigIntObj>>,

    #[prop("RegExp")]
    regexp: Partial<ObjectHandle, GlobalInitializer<RegExp>>,

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
    array_buffer: Partial<ObjectHandle, GlobalInitializer<ArrayBuffer>>,

    #[prop("SharedArrayBuffer")]
    shared_array_buffer: Partial<ObjectHandle, GlobalInitializer<SharedArrayBuffer>>,

    #[prop("DataView")]
    data_view: Partial<ObjectHandle, GlobalInitializer<DataView>>,

    #[prop("Int8Array")]
    int8_array: Partial<ObjectHandle, GlobalInitializer<Int8Array>>,

    #[prop("Uint8Array")]
    uint8_array: Partial<ObjectHandle, GlobalInitializer<Uint8Array>>,

    #[prop("Uint8ClampedArray")]
    uint8_clamped_array: Partial<ObjectHandle, GlobalInitializer<Uint8ClampedArray>>,

    #[prop("Int16Array")]
    int16_array: Partial<ObjectHandle, GlobalInitializer<Int16Array>>,

    #[prop("Uint16Array")]
    uint16_array: Partial<ObjectHandle, GlobalInitializer<Uint16Array>>,

    #[prop("Int32Array")]
    int32_array: Partial<ObjectHandle, GlobalInitializer<Int32Array>>,

    #[prop("Uint32Array")]
    uint32_array: Partial<ObjectHandle, GlobalInitializer<Uint32Array>>,

    #[prop("Float16Array")]
    float16_array: Partial<ObjectHandle, GlobalInitializer<Float16Array>>,

    #[prop("Float32Array")]
    float32_array: Partial<ObjectHandle, GlobalInitializer<Float32Array>>,

    #[prop("Float64Array")]
    float64_array: Partial<ObjectHandle, GlobalInitializer<Float64Array>>,

    #[prop("BigInt64Array")]
    bigint64_array: Partial<ObjectHandle, GlobalInitializer<BigInt64Array>>,

    #[prop("BigUint64Array")]
    biguint64_array: Partial<ObjectHandle, GlobalInitializer<BigUint64Array>>,

    #[prop("Atomics")]
    atomics: Partial<ObjectHandle, GlobalInitializer<Atomics>>,

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
    map: Partial<ObjectHandle, GlobalInitializer<Map>>,

    #[prop("WeakMap")]
    weak_map: Partial<ObjectHandle, GlobalInitializer<WeakMap>>,

    #[prop("WeakRef")]
    weak_ref: Partial<ObjectHandle, GlobalInitializer<WeakRef>>,

    #[prop("Set")]
    set: Partial<ObjectHandle, GlobalInitializer<Set>>,

    #[prop("WeakSet")]
    weak_set: Partial<ObjectHandle, GlobalInitializer<WeakSet>>,

    #[prop("Date")]
    date: Partial<ObjectHandle, GlobalInitializer<Date>>,

    #[prop("Reflect")]
    reflect: ObjectHandle,

    #[prop("Proxy")]
    proxy: Partial<ObjectHandle, GlobalInitializer<Proxy>>,

    #[prop("Temporal")]
    temporal: ObjectHandle,

    #[prop("Signal")]
    signal: ObjectHandle,

    #[prop("Promise")]
    promise: Partial<ObjectHandle, GlobalInitializer<Promise>>,

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
        error: Default::default(),
        array: Default::default(),
        object: RefCell::new(realm.intrinsics.obj_constructor()),
        function: RefCell::new(realm.intrinsics.func_constructor()),
        math: RefCell::new(realm.intrinsics.math_obj()),
        string: Default::default(),
        number: Default::default(),
        boolean: Default::default(),
        symbol: Default::default(),
        bigint: Default::default(),
        regexp: Default::default(),
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
        array_buffer: Default::default(),
        shared_array_buffer: Default::default(),
        data_view: Default::default(),
        int8_array: Default::default(),
        uint8_array: Default::default(),
        uint8_clamped_array: Default::default(),
        int16_array: Default::default(),
        uint16_array: Default::default(),
        int32_array: Default::default(),
        uint32_array: Default::default(),
        float16_array: Default::default(),
        float32_array: Default::default(),
        float64_array: Default::default(),
        bigint64_array: Default::default(),
        biguint64_array: Default::default(),
        atomics: Default::default(),
        escape: RefCell::new(get_escape(realm)),
        unescape: RefCell::new(get_escape(realm)),
        encode_uri: RefCell::new(get_encode_uri(realm)),
        decode_uri: RefCell::new(get_decode_uri(realm)),
        encode_uri_component: RefCell::new(get_encode_uri_component(realm)),
        decode_uri_component: RefCell::new(get_decode_uri_component(realm)),
        map: Default::default(),
        weak_map: Default::default(),
        weak_ref: Default::default(),
        set: Default::default(),
        weak_set: Default::default(),
        date: Default::default(),
        reflect: RefCell::new(realm.intrinsics.reflect_obj()),
        proxy: Default::default(),
        temporal: RefCell::new(realm.intrinsics.temporal_obj()),
        signal: RefCell::new(realm.intrinsics.signal_obj()),
        promise: Default::default(),
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

#[derive(Debug)]
pub struct GlobalInitializer<T> {
    marker: std::marker::PhantomData<T>,
}

impl<T: Intrinsic> Initializer<ObjectHandle> for GlobalInitializer<T> {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        T::get_global(realm)
    }
}
