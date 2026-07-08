use crate::array::Array;
#[cfg(feature = "temporal")]
use crate::builtins::Temporal;
use crate::builtins::array_buf::ArrayBuffer;
use crate::builtins::bigint64array::BigInt64Array;
use crate::builtins::biguint64array::BigUint64Array;
use crate::builtins::dataview::DataView;
use crate::builtins::float16array::Float16Array;
use crate::builtins::float32array::Float32Array;
use crate::builtins::float64array::Float64Array;
use crate::builtins::int8array::Int8Array;
use crate::builtins::int16array::Int16Array;
use crate::builtins::int32array::Int32Array;
#[cfg(feature = "icu")]
use crate::builtins::intl::Intl;
use crate::builtins::iterator::Iterator as IteratorConstructor;
use crate::builtins::shared_buf::SharedArrayBuffer;
use crate::builtins::signal::Signal;
use crate::builtins::uint8clampedarray::Uint8ClampedArray;
use crate::builtins::uint16array::Uint16Array;
use crate::builtins::uint32array::Uint32Array;
use crate::builtins::unit8array::Uint8Array;
use crate::builtins::{
    AggregateError, AsyncDisposableStack, Atomics, BigIntObj, BooleanObj, Date, DecodeURI,
    DecodeURIComponent, DisposableStack, EncodeURI, EncodeURIComponent, Escape, EvalError,
    IsFinite, IsNan, JSON, Map, Math, NumberObj, Promise, Proxy, RangeError, ReferenceError,
    Reflect, RegExp, Set, StringObj, SuppressedError, SymbolObj, SyntaxError, TypeError, URIError,
    Unescape, WeakMap, WeakRef, WeakSet,
};
use crate::error_obj::ErrorObj;
use crate::function::function_prototype::GlobalFunctionConstructor;
use crate::inline_props::InlineObject;
use crate::object::prototype::GlobalObjectConstructor;
use crate::partial_init::{Initializer, Partial};
use crate::realm::{Intrinsic, Realm};
use crate::value::Obj;
use crate::{Console, Value};
use crate::{ObjectHandle, Res};
use std::cell::Cell;
use yavashark_macro::inline_props;

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

    console: Partial<ObjectHandle, Console>,

    #[prop("Error")]
    error: Partial<ObjectHandle, GlobalInitializer<ErrorObj>>,

    #[prop("Array")]
    array: Partial<ObjectHandle, GlobalInitializer<Array>>,

    #[prop("Object")]
    object: Partial<ObjectHandle, GlobalObjectConstructor>,

    #[prop("Function")]
    function: Partial<ObjectHandle, GlobalFunctionConstructor>,

    #[prop("Math")]
    math: Partial<ObjectHandle, Math>,

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
    json: Partial<ObjectHandle, JSON>,

    #[prop("TypeError")]
    type_error: Partial<ObjectHandle, GlobalInitializer<TypeError>>,

    #[prop("RangeError")]
    range_error: Partial<ObjectHandle, GlobalInitializer<RangeError>>,

    #[prop("ReferenceError")]
    reference_error: Partial<ObjectHandle, GlobalInitializer<ReferenceError>>,

    #[prop("SyntaxError")]
    syntax_error: Partial<ObjectHandle, GlobalInitializer<SyntaxError>>,

    #[prop("EvalError")]
    eval_error: Partial<ObjectHandle, GlobalInitializer<EvalError>>,

    #[prop("URIError")]
    uri_error: Partial<ObjectHandle, GlobalInitializer<URIError>>,

    #[prop("AggregateError")]
    aggregate_error: Partial<ObjectHandle, GlobalInitializer<AggregateError>>,

    #[prop("SuppressedError")]
    suppressed_error: Partial<ObjectHandle, GlobalInitializer<SuppressedError>>,

    #[prop("globalThis")]
    global_this: Partial<ObjectHandle, GlobalThis>,

    global: Partial<ObjectHandle, GlobalThis>,

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

    escape: Partial<ObjectHandle, Escape>,

    unescape: Partial<ObjectHandle, Unescape>,

    #[prop("encodeURI")]
    encode_uri: Partial<ObjectHandle, EncodeURI>,

    #[prop("decodeURI")]
    decode_uri: Partial<ObjectHandle, DecodeURI>,

    #[prop("encodeURIComponent")]
    encode_uri_component: Partial<ObjectHandle, EncodeURIComponent>,

    #[prop("decodeURIComponent")]
    decode_uri_component: Partial<ObjectHandle, DecodeURIComponent>,

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
    reflect: Partial<ObjectHandle, Reflect>,

    #[prop("Proxy")]
    proxy: Partial<ObjectHandle, GlobalInitializer<Proxy>>,

    #[cfg(feature = "temporal")]
    #[prop("Temporal")]
    temporal: Partial<ObjectHandle, Temporal>,

    #[prop("Signal")]
    signal: Partial<ObjectHandle, Signal>,

    #[prop("Promise")]
    promise: Partial<ObjectHandle, GlobalInitializer<Promise>>,

    #[prop("Iterator")]
    iterator: Partial<ObjectHandle, GlobalInitializer<IteratorConstructor>>,

    #[prop("parseInt")]
    parse_int: Partial<ObjectHandle, IntrinsicParseInt>,

    #[prop("parseFloat")]
    parse_float: Partial<ObjectHandle, IntrinsicParseFloat>,

    #[prop("isNaN")]
    is_nan: Partial<ObjectHandle, IsNan>,

    #[prop("isFinite")]
    is_finite: Partial<ObjectHandle, IsFinite>,

    #[cfg(feature = "icu")]
    #[prop("Intl")]
    intl: Partial<ObjectHandle, Intl>,

    #[prop("DisposableStack")]
    disposable_stack: Partial<ObjectHandle, GlobalInitializer<DisposableStack>>,

    #[prop("AsyncDisposableStack")]
    async_disposable_stack: Partial<ObjectHandle, GlobalInitializer<AsyncDisposableStack>>,
}

pub fn new_global_obj(proto: ObjectHandle) -> Res<ObjectHandle> {
    let inline = GlobalProperties {
        undefined: (),
        nan: f64::NAN,
        infinity: f64::INFINITY,
        null: Value::Null,
        true_: true,
        false_: false,
        console: Partial::default(),
        error: Partial::default(),
        array: Partial::default(),
        object: Partial::default(),
        function: Partial::default(),
        math: Partial::default(),
        string: Partial::default(),
        number: Partial::default(),
        boolean: Partial::default(),
        symbol: Partial::default(),
        bigint: Partial::default(),
        regexp: Partial::default(),
        json: Partial::default(),
        type_error: Partial::default(),
        range_error: Partial::default(),
        reference_error: Partial::default(),
        syntax_error: Partial::default(),
        eval_error: Partial::default(),
        uri_error: Partial::default(),
        aggregate_error: Partial::default(),
        suppressed_error: Partial::default(),
        global_this: Partial::default(),
        global: Partial::default(),
        array_buffer: Partial::default(),
        shared_array_buffer: Partial::default(),
        data_view: Partial::default(),
        int8_array: Partial::default(),
        uint8_array: Partial::default(),
        uint8_clamped_array: Partial::default(),
        int16_array: Partial::default(),
        uint16_array: Partial::default(),
        int32_array: Partial::default(),
        uint32_array: Partial::default(),
        float16_array: Partial::default(),
        float32_array: Partial::default(),
        float64_array: Partial::default(),
        bigint64_array: Partial::default(),
        biguint64_array: Partial::default(),
        atomics: Partial::default(),
        escape: Partial::default(),
        unescape: Partial::default(),
        encode_uri: Partial::default(),
        decode_uri: Partial::default(),
        encode_uri_component: Partial::default(),
        decode_uri_component: Partial::default(),
        map: Partial::default(),
        weak_map: Partial::default(),
        weak_ref: Partial::default(),
        set: Partial::default(),
        weak_set: Partial::default(),
        date: Partial::default(),
        reflect: Partial::default(),
        proxy: Partial::default(),
        #[cfg(feature = "temporal")]
        temporal: Partial::default(),
        signal: Partial::default(),
        promise: Partial::default(),
        iterator: Partial::default(),
        parse_int: Partial::default(),
        parse_float: Partial::default(),
        is_nan: Partial::default(),
        is_finite: Partial::default(),
        #[cfg(feature = "icu")]
        intl: Partial::default(),
        disposable_stack: Partial::default(),
        async_disposable_stack: Partial::default(),

        __deleted_properties: Cell::default(),
        __written_properties: Cell::default(),
    };

    let handle = InlineObject::with_proto(inline, proto).into_object();

    Ok(handle)
}

#[allow(unused)]
#[inline(always)]
pub fn init_global_obj(realm: &mut Realm) -> Res<()> {
    #[cfg(feature = "out-of-spec-experiments")]
    crate::experiments::init(&realm.global.clone(), realm)?;

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

pub struct GlobalThis;

impl Initializer<ObjectHandle> for GlobalThis {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(realm.global.clone())
    }
}

/// Initializer that gets parseInt from intrinsics
pub struct IntrinsicParseInt;

impl Initializer<ObjectHandle> for IntrinsicParseInt {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        realm
            .intrinsics
            .clone_public()
            .parse_int
            .get(realm)
            .cloned()
    }
}

/// Initializer that gets parseFloat from intrinsics
pub struct IntrinsicParseFloat;

impl Initializer<ObjectHandle> for IntrinsicParseFloat {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        realm
            .intrinsics
            .clone_public()
            .parse_float
            .get(realm)
            .cloned()
    }
}
