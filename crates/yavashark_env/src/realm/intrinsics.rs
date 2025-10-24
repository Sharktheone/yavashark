use crate::array::{Array, ArrayIterator};
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
use crate::builtins::typed_array::TypedArray;
use crate::builtins::uint16array::Uint16Array;
use crate::builtins::uint32array::Uint32Array;
use crate::builtins::uint8clampedarray::Uint8ClampedArray;
use crate::builtins::unit8array::Uint8Array;
use crate::builtins::{
    intl, signal, temporal, AggregateError, Arguments, Atomics, BigIntObj, BooleanObj, Date,
    EvalError, Map, NumberObj, Promise, Proxy, RangeError, ReferenceError, RegExp, Set, StringObj,
    SymbolObj, SyntaxError, ThrowTypeError, TypeError, URIError, WeakMap, WeakRef, WeakSet,
};
use crate::error_obj::ErrorObj;
use crate::partial_init::Partial;
use crate::realm::initialize::Intrinsic;
use crate::value::Obj;
use crate::{Error, FunctionPrototype, Object, ObjectHandle, Prototype, Realm, Res};
use rustc_hash::FxHashMap;
use std::any::TypeId;

type PartialIntrinsic<T> = Partial<ObjectHandle, IntrinsicInitializer<T>>;

pub struct Intrinsics {
    pub obj: ObjectHandle,
    pub func: ObjectHandle,
    pub array: PartialIntrinsic<Array>,
    pub array_iter: PartialIntrinsic<ArrayIterator>,
    pub error: PartialIntrinsic<ErrorObj>,
    pub string: PartialIntrinsic<StringObj>,
    pub number: PartialIntrinsic<NumberObj>,
    pub boolean: PartialIntrinsic<BooleanObj>,
    pub symbol: PartialIntrinsic<SymbolObj>,
    pub bigint: PartialIntrinsic<BigIntObj>,
    pub regexp: PartialIntrinsic<RegExp>,
    pub ty_error: PartialIntrinsic<TypeError>,
    pub range_error: PartialIntrinsic<RangeError>,
    pub reference_error: PartialIntrinsic<ReferenceError>,
    pub syn_error: PartialIntrinsic<SyntaxError>,
    pub eval_error: PartialIntrinsic<EvalError>,
    pub uri_error: PartialIntrinsic<URIError>,
    pub aggregate_error: PartialIntrinsic<AggregateError>,
    pub eval: Option<ObjectHandle>,
    pub arraybuffer: PartialIntrinsic<ArrayBuffer>,
    pub sharedarraybuffer: PartialIntrinsic<SharedArrayBuffer>,
    pub data_view: PartialIntrinsic<DataView>,
    pub typed_array: PartialIntrinsic<TypedArray>,
    pub int8array: PartialIntrinsic<Int8Array>,
    pub uint8array: PartialIntrinsic<Uint8Array>,
    pub uint8clampedarray: PartialIntrinsic<Uint8ClampedArray>,
    pub int16array: PartialIntrinsic<Int16Array>,
    pub uint16array: PartialIntrinsic<Uint16Array>,
    pub int32array: PartialIntrinsic<Int32Array>,
    pub uint32array: PartialIntrinsic<Uint32Array>,
    pub float16array: PartialIntrinsic<Float16Array>,
    pub float32array: PartialIntrinsic<Float32Array>,
    pub float64array: PartialIntrinsic<Float64Array>,
    pub bigint64array: PartialIntrinsic<BigInt64Array>,
    pub biguint64array: PartialIntrinsic<BigUint64Array>,
    pub atomics: PartialIntrinsic<Atomics>,
    pub map: PartialIntrinsic<Map>,
    pub weak_map: PartialIntrinsic<WeakMap>,
    pub set: PartialIntrinsic<Set>,
    pub weak_set: PartialIntrinsic<WeakSet>,
    pub weak_ref: PartialIntrinsic<WeakRef>,
    pub date: PartialIntrinsic<Date>,
    pub temporal_duration: PartialIntrinsic<temporal::Duration>,
    pub temporal_instant: PartialIntrinsic<temporal::Instant>,
    pub temporal_now: PartialIntrinsic<temporal::Now>,
    pub temporal_plain_date: PartialIntrinsic<temporal::PlainDate>,
    pub temporal_plain_time: PartialIntrinsic<temporal::PlainTime>,
    pub temporal_plain_date_time: PartialIntrinsic<temporal::PlainDateTime>,
    pub temporal_plain_month_day: PartialIntrinsic<temporal::PlainMonthDay>,
    pub temporal_plain_year_month: PartialIntrinsic<temporal::PlainYearMonth>,
    pub temporal_zoned_date_time: PartialIntrinsic<temporal::ZonedDateTime>,
    pub promise: PartialIntrinsic<Promise>,
    pub generator_function: Partial<ObjectHandle, NullObjInitializer>,
    pub generator: Partial<ObjectHandle, NullObjInitializer>,
    pub async_generator: Partial<ObjectHandle, NullObjInitializer>,
    pub async_generator_function: Partial<ObjectHandle, NullObjInitializer>,
    pub signal_state: PartialIntrinsic<signal::State>,
    pub signal_computed: PartialIntrinsic<signal::Computed>,
    pub arguments: PartialIntrinsic<Arguments>,
    pub proxy: PartialIntrinsic<Proxy>,
    pub intl_collator: PartialIntrinsic<intl::Collator>,
    pub intl_date_time_format: PartialIntrinsic<intl::DateTimeFormat>,
    pub intl_display_names: PartialIntrinsic<intl::DisplayNames>,
    pub intl_duration_format: PartialIntrinsic<intl::DurationFormat>,
    pub intl_list_format: PartialIntrinsic<intl::ListFormat>,
    pub intl_locale: PartialIntrinsic<intl::Locale>,
    pub intl_number_format: PartialIntrinsic<intl::NumberFormat>,
    pub intl_plural_rules: PartialIntrinsic<intl::PluralRules>,
    pub intl_relative_time_format: PartialIntrinsic<intl::RelativeTimeFormat>,
    pub intl_segmenter: PartialIntrinsic<intl::Segmenter>,
    pub throw_type_error: Partial<ObjectHandle, ThrowTypeError>,

    pub other: FxHashMap<TypeId, ObjectHandle>,
}

#[allow(clippy::similar_names)]
impl Intrinsics {
    pub(crate) fn initialize(realm: &mut Realm) -> Res {
        realm.intrinsics.obj = Prototype::new().into_object();
        realm.intrinsics.func = FunctionPrototype::new(realm).into_object();

        Ok(())
    }

    pub fn get_of<T: 'static>(&self) -> Res<ObjectHandle> {
        self.other
            .get(&TypeId::of::<T>())
            .cloned()
            .ok_or(Error::new("Failed to get prototype"))
    }

    pub fn insert<T: 'static>(&mut self, proto: ObjectHandle) {
        self.other.insert(TypeId::of::<T>(), proto);
    }
}

impl Default for Intrinsics {
    fn default() -> Self {
        Self {
            obj: Object::null(),
            func: Object::null(),
            array: Default::default(),
            array_iter: Default::default(),
            error: Default::default(),
            string: Default::default(),
            number: Default::default(),
            boolean: Default::default(),
            symbol: Default::default(),
            bigint: Default::default(),
            regexp: Default::default(),
            ty_error: Default::default(),
            range_error: Default::default(),
            reference_error: Default::default(),
            syn_error: Default::default(),
            eval_error: Default::default(),
            uri_error: Default::default(),
            aggregate_error: Default::default(),
            eval: None,
            arraybuffer: Default::default(),
            sharedarraybuffer: Default::default(),
            data_view: Default::default(),
            typed_array: Default::default(),
            int8array: Default::default(),
            uint8array: Default::default(),
            uint8clampedarray: Default::default(),
            int16array: Default::default(),
            uint16array: Default::default(),
            int32array: Default::default(),
            uint32array: Default::default(),
            float16array: Default::default(),
            float32array: Default::default(),
            float64array: Default::default(),
            bigint64array: Default::default(),
            biguint64array: Default::default(),
            atomics: Default::default(),
            map: Default::default(),
            weak_map: Default::default(),
            set: Default::default(),
            weak_set: Default::default(),
            weak_ref: Default::default(),
            date: Default::default(),
            temporal_duration: Default::default(),
            temporal_instant: Default::default(),
            temporal_now: Default::default(),
            temporal_plain_date: Default::default(),
            temporal_plain_time: Default::default(),
            temporal_plain_date_time: Default::default(),
            temporal_plain_month_day: Default::default(),
            temporal_plain_year_month: Default::default(),
            temporal_zoned_date_time: Default::default(),
            promise: Default::default(),
            generator_function: Default::default(),
            generator: Default::default(),
            async_generator_function: Default::default(),
            async_generator:  Default::default(),
            signal_state: Default::default(),
            signal_computed: Default::default(),
            arguments: Default::default(),
            proxy: Default::default(),
            intl_collator: Default::default(),
            intl_date_time_format: Default::default(),
            intl_display_names: Default::default(),
            intl_duration_format: Default::default(),
            intl_list_format: Default::default(),
            intl_locale: Default::default(),
            intl_number_format: Default::default(),
            intl_plural_rules: Default::default(),
            intl_relative_time_format: Default::default(),
            intl_segmenter: Default::default(),
            throw_type_error: Default::default(),
            other: FxHashMap::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntrinsicInitializer<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: Intrinsic> crate::partial_init::Initializer<ObjectHandle> for IntrinsicInitializer<T> {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        T::initialize(realm)
    }
}


pub struct NullObjInitializer;

impl crate::partial_init::Initializer<ObjectHandle> for NullObjInitializer {
    fn initialize(_realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(Object::null())
    }
}