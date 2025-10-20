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
use crate::builtins::{get_aggregate_error, get_eval_error, get_range_error, get_reference_error, get_syntax_error, get_throw_type_error, get_type_error, get_uri_error, intl, signal, temporal, Arguments, Atomics, BigIntObj, BooleanObj, Date, Map, Math, NumberObj, Promise, Proxy, Reflect, RegExp, Set, StringObj, SymbolObj, WeakMap, WeakRef, WeakSet, JSON};
use crate::error_obj::ErrorObj;
use crate::{
    Error, FunctionPrototype, Object, ObjectHandle, Prototype, Realm, Res, Value,
};
use rustc_hash::FxHashMap;
use std::any::TypeId;
use crate::partial_init::Partial;
use crate::realm::initialize::Intrinsic;

type PartialIntrinsic<T> = Partial<ObjectHandle, IntrinsicInitializer<T>>;


pub struct Intrinsics {
    pub obj: ObjectHandle,
    pub func: ObjectHandle,
    pub array: PartialIntrinsic<Array>,
    pub array_iter: ObjectHandle,
    pub error: PartialIntrinsic<ErrorObj>,
    pub math: ObjectHandle,
    pub string: PartialIntrinsic<StringObj>,
    pub number: PartialIntrinsic<NumberObj>,
    pub boolean: PartialIntrinsic<BooleanObj>,
    pub symbol: PartialIntrinsic<SymbolObj>,
    pub bigint: PartialIntrinsic<BigIntObj>,
    pub regexp: PartialIntrinsic<RegExp>,
    pub json: ObjectHandle,
    pub type_error: ObjectHandle,
    pub range_error: ObjectHandle,
    pub reference_error: ObjectHandle,
    pub syntax_error: ObjectHandle,
    pub eval_error: ObjectHandle,
    pub uri_error: ObjectHandle,
    pub aggregate_error: ObjectHandle,
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
    pub reflect: ObjectHandle,
    pub temporal: ObjectHandle,
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
    pub generator_function: ObjectHandle,
    pub generator: ObjectHandle,
    pub async_generator: ObjectHandle,
    pub async_generator_function: ObjectHandle,
    pub signal: ObjectHandle,
    pub signal_state: PartialIntrinsic<signal::State>,
    pub signal_computed: PartialIntrinsic<signal::Computed>,
    pub arguments: PartialIntrinsic<Arguments>,
    pub proxy: PartialIntrinsic<Proxy>,
    pub intl: ObjectHandle,
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
    pub throw_type_error: ObjectHandle,

    pub other: FxHashMap<TypeId, ObjectHandle>,
}

macro_rules! constructor {
    ($name:ident) => {
        paste::paste! {
            pub fn [<$name _constructor>] (&self) -> ObjectHandle {
                self.$name
                    .resolve_property("constructor", &mut Realm::default()) //TODO: this is bad, but we don't have access to a realm here
                    .ok()
                    .flatten()
                    .map(|v| v.to_object().unwrap_or(Object::null()))
                    .unwrap_or(Object::null())
            }
        }
    };
}

macro_rules! obj {
    ($name:ident) => {
        paste::paste! {
            pub fn [<$name _obj>] (&self) -> ObjectHandle {
                self.$name.clone()
            }
        }
    };
}

impl Intrinsics {
    constructor!(obj);
    constructor!(func);
//     constructor!(array);
//     constructor!(array_iter);
//     constructor!(error);
//     constructor!(string);
//     constructor!(number);
//     constructor!(boolean);
//     constructor!(symbol);
//     constructor!(bigint);
//     constructor!(regexp);
    constructor!(type_error);
    constructor!(range_error);
    constructor!(reference_error);
    constructor!(syntax_error);
    constructor!(eval_error);
    constructor!(uri_error);
    constructor!(aggregate_error);
//     constructor!(arraybuffer);
//     constructor!(sharedarraybuffer);
//     constructor!(data_view);
//     constructor!(typed_array);
//     constructor!(int8array);
//     constructor!(uint8array);
//     constructor!(uint8clampedarray);
//     constructor!(int16array);
//     constructor!(uint16array);
//     constructor!(int32array);
//     constructor!(uint32array);
//     constructor!(float16array);
//     constructor!(float32array);
//     constructor!(float64array);
//     constructor!(bigint64array);
//     constructor!(biguint64array);
//     constructor!(map);
//     constructor!(weak_map);
//     constructor!(set);
//     constructor!(weak_set);
//     constructor!(weak_ref);
//     constructor!(date);
//     constructor!(promise);
//     constructor!(proxy);
//     constructor!(atomics);
//
    obj!(json);
    obj!(math);
    obj!(reflect);
    obj!(temporal);
    obj!(signal);
    obj!(intl);
}

#[allow(clippy::similar_names)]
impl Intrinsics {
    pub(crate) fn initialize(realm: &mut Realm) -> Res {
        realm.intrinsics.obj = ObjectHandle::new(Prototype::new());

        realm.intrinsics.func = ObjectHandle::new(FunctionPrototype::new(realm.intrinsics.obj.clone()));

        {
            let obj_this = realm.intrinsics.obj.clone();
            let obj_this2 = obj_this.clone();
            let obj = obj_this.guard();

            let proto = obj
                .downcast::<Prototype>()
                .ok_or_else(|| Error::new("downcast_mut::<Prototype> failed"))?;

            proto.initialize(realm.intrinsics.func.clone(), obj_this2, realm)?;
        }

        {
            let func = realm.intrinsics.func.clone();
            let func = func.guard();

            let proto = func
                .downcast::<FunctionPrototype>()
                .ok_or_else(|| Error::new("downcast_mut::<FunctionPrototype> failed"))?;

            proto.initialize(realm.intrinsics.func.clone(), realm)?;
        }

        realm.intrinsics.array_iter = ArrayIterator::initialize_proto(
            Object::raw_with_proto(realm.intrinsics.obj.clone()),
            realm.intrinsics.func.clone(),
            realm,
        )?;

        realm.intrinsics.math = Math::new(realm)?;

        realm.intrinsics.json = JSON::new(realm)?;

        let error = realm.intrinsics.clone_public().error.get(realm)?.clone();

        let error_constructor = error
            .resolve_property("constructor", realm)
            .unwrap_or(Value::Undefined.into())
            .unwrap_or(Value::Undefined.into())
            .to_object()?;

        realm.intrinsics.type_error =
            get_type_error(error.clone(), error_constructor.clone(), realm)?;
        realm.intrinsics.range_error =
            get_range_error(error.clone(), error_constructor.clone(), realm)?;
        realm.intrinsics.reference_error =
            get_reference_error(error.clone(), error_constructor.clone(), realm)?;
        realm.intrinsics.syntax_error =
            get_syntax_error(error.clone(), error_constructor.clone(), realm)?;

        realm.intrinsics.eval_error =
            get_eval_error(error.clone(), error_constructor.clone(), realm)?;

        realm.intrinsics.uri_error =
            get_uri_error(error.clone(), error_constructor.clone(), realm)?;

        realm.intrinsics.aggregate_error =
            get_aggregate_error(error.clone(), error_constructor, realm)?;

        realm.intrinsics.reflect = Reflect::new(realm)?;

        realm.intrinsics.temporal = temporal::get_temporal(realm)?;


        realm.intrinsics.intl = intl::get_intl(realm)?;

        realm.intrinsics.signal = signal::get_signal(realm)?;

        realm.intrinsics.throw_type_error = get_throw_type_error(realm)?;

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
            array_iter: Object::null(),
            error: Default::default(),
            math: Object::null(),
            string: Default::default(),
            number: Default::default(),
            boolean: Default::default(),
            symbol: Default::default(),
            bigint: Default::default(),
            regexp: Default::default(),
            json: Object::null(),
            type_error: Object::null(),
            range_error: Object::null(),
            reference_error: Object::null(),
            syntax_error: Object::null(),
            eval_error: Object::null(),
            uri_error: Object::null(),
            aggregate_error: Object::null(),
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
            reflect: Object::null(),
            temporal: Object::null(),
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
            generator_function: Object::null(),
            generator: Object::null(),
            async_generator_function: Object::null(),
            async_generator: Object::null(),
            signal: Object::null(),
            signal_state: Default::default(),
            signal_computed: Default::default(),
            arguments: Default::default(),
            proxy: Default::default(),
            intl: Object::null(),
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
            throw_type_error: Object::null(),
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
