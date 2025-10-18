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
use crate::builtins::{get_aggregate_error, get_eval_error, get_range_error, get_reference_error, get_syntax_error, get_temporal, get_throw_type_error, get_type_error, get_uri_error, Arguments, Atomics, BigIntObj, BooleanObj, Date, Map, Math, NumberObj, Promise, Proxy, Reflect, RegExp, Set, StringObj, SymbolObj, WeakMap, WeakRef, WeakSet, JSON};
use crate::error_obj::ErrorObj;
use crate::{
    Error, FunctionPrototype, Object, ObjectHandle, Prototype, Realm, Res, Value, Variable,
};
use rustc_hash::FxHashMap;
use std::any::TypeId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Intrinsics {
    pub obj: ObjectHandle,
    pub func: ObjectHandle,
    pub array: ObjectHandle,
    pub array_iter: ObjectHandle,
    pub error: ObjectHandle,
    pub math: ObjectHandle,
    pub string: ObjectHandle,
    pub number: ObjectHandle,
    pub boolean: ObjectHandle,
    pub symbol: ObjectHandle,
    pub bigint: ObjectHandle,
    pub regexp: ObjectHandle,
    pub json: ObjectHandle,
    pub type_error: ObjectHandle,
    pub range_error: ObjectHandle,
    pub reference_error: ObjectHandle,
    pub syntax_error: ObjectHandle,
    pub eval_error: ObjectHandle,
    pub uri_error: ObjectHandle,
    pub aggregate_error: ObjectHandle,
    pub eval: Option<ObjectHandle>,
    pub arraybuffer: ObjectHandle,
    pub sharedarraybuffer: ObjectHandle,
    pub data_view: ObjectHandle,
    pub typed_array: ObjectHandle,
    pub int8array: ObjectHandle,
    pub uint8array: ObjectHandle,
    pub uint8clampedarray: ObjectHandle,
    pub int16array: ObjectHandle,
    pub uint16array: ObjectHandle,
    pub int32array: ObjectHandle,
    pub uint32array: ObjectHandle,
    pub float16array: ObjectHandle,
    pub float32array: ObjectHandle,
    pub float64array: ObjectHandle,
    pub bigint64array: ObjectHandle,
    pub biguint64array: ObjectHandle,
    pub atomics: ObjectHandle,
    pub map: ObjectHandle,
    pub weak_map: ObjectHandle,
    pub set: ObjectHandle,
    pub weak_set: ObjectHandle,
    pub weak_ref: ObjectHandle,
    pub date: ObjectHandle,
    pub reflect: ObjectHandle,
    pub temporal: ObjectHandle,
    pub temporal_duration: ObjectHandle,
    pub temporal_instant: ObjectHandle,
    pub temporal_now: ObjectHandle,
    pub temporal_plain_date: ObjectHandle,
    pub temporal_plain_time: ObjectHandle,
    pub temporal_plain_date_time: ObjectHandle,
    pub temporal_plain_month_day: ObjectHandle,
    pub temporal_plain_year_month: ObjectHandle,
    pub temporal_zoned_date_time: ObjectHandle,
    pub promise: ObjectHandle,
    pub generator_function: ObjectHandle,
    pub generator: ObjectHandle,
    pub async_generator: ObjectHandle,
    pub async_generator_function: ObjectHandle,
    pub signal: ObjectHandle,
    pub signal_state: ObjectHandle,
    pub signal_computed: ObjectHandle,
    pub arguments: ObjectHandle,
    pub proxy: ObjectHandle,
    pub intl: ObjectHandle,
    pub intl_collator: ObjectHandle,
    pub intl_date_time_format: ObjectHandle,
    pub intl_display_names: ObjectHandle,
    pub intl_duration_format: ObjectHandle,
    pub intl_list_format: ObjectHandle,
    pub intl_locale: ObjectHandle,
    pub intl_number_format: ObjectHandle,
    pub intl_plural_rules: ObjectHandle,
    pub intl_relative_time_format: ObjectHandle,
    pub intl_segmenter: ObjectHandle,
    pub throw_type_error: ObjectHandle,

    pub other: FxHashMap<TypeId, ObjectHandle>,
}

macro_rules! constructor {
    ($name:ident) => {
        paste::paste! {
            pub fn [<$name _constructor>] (&self) -> Variable {
                self.$name
                    .resolve_property("constructor", &mut Realm::default()) //TODO: this is bad, but we don't have access to a realm here
                    .unwrap_or(Value::Undefined.into())
                    .unwrap_or(Value::Undefined.into())
                    .into()
            }
        }
    };
}

macro_rules! obj {
    ($name:ident) => {
        paste::paste! {
            pub fn [<$name _obj>] (&self) -> Variable {
                self.$name.clone().into()
            }
        }
    };
}

impl Intrinsics {
    constructor!(obj);
    constructor!(func);
    constructor!(array);
    constructor!(array_iter);
    constructor!(error);
    constructor!(string);
    constructor!(number);
    constructor!(boolean);
    constructor!(symbol);
    constructor!(bigint);
    constructor!(regexp);
    constructor!(type_error);
    constructor!(range_error);
    constructor!(reference_error);
    constructor!(syntax_error);
    constructor!(eval_error);
    constructor!(uri_error);
    constructor!(aggregate_error);
    constructor!(arraybuffer);
    constructor!(sharedarraybuffer);
    constructor!(data_view);
    constructor!(typed_array);
    constructor!(int8array);
    constructor!(uint8array);
    constructor!(uint8clampedarray);
    constructor!(int16array);
    constructor!(uint16array);
    constructor!(int32array);
    constructor!(uint32array);
    constructor!(float16array);
    constructor!(float32array);
    constructor!(float64array);
    constructor!(bigint64array);
    constructor!(biguint64array);
    constructor!(map);
    constructor!(weak_map);
    constructor!(set);
    constructor!(weak_set);
    constructor!(weak_ref);
    constructor!(date);
    constructor!(promise);
    constructor!(proxy);
    constructor!(atomics);

    obj!(json);
    obj!(math);
    obj!(reflect);
    obj!(temporal);
    obj!(signal);
    obj!(intl);
}

#[allow(clippy::similar_names)]
impl Intrinsics {
    pub(crate) fn new() -> Result<Self, Error> {
        let realm = &mut Realm::default();

        let obj_prototype = ObjectHandle::new(Prototype::new());

        let func_prototype =
            ObjectHandle::new(FunctionPrototype::new(obj_prototype.clone().into()));

        {
            let obj_this = obj_prototype.clone().into();
            let obj = obj_prototype.guard();

            let proto = obj
                .downcast::<Prototype>()
                .ok_or_else(|| Error::new("downcast_mut::<Prototype> failed"))?;

            proto.initialize(func_prototype.clone().into(), obj_this, realm)?;
        }

        {
            let func = func_prototype.guard();

            let proto = func
                .downcast::<FunctionPrototype>()
                .ok_or_else(|| Error::new("downcast_mut::<FunctionPrototype> failed"))?;

            proto.initialize(func_prototype.clone().into(), realm)?;
        }

        let array_prototype = Array::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let array_iter_prototype = ArrayIterator::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let error_prototype = ErrorObj::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let math_obj = Math::new(obj_prototype.clone(), func_prototype.clone(), realm)?;

        let string_prototype = StringObj::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let number_prototype = NumberObj::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let boolean_prototype = BooleanObj::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let symbol_prototype = SymbolObj::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let bigint_prototype = BigIntObj::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let regex = RegExp::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let json = JSON::new(obj_prototype.clone(), func_prototype.clone(), realm)?;

        let error_constructor = error_prototype
            .resolve_property("constructor", realm)
            .unwrap_or(Value::Undefined.into())
            .unwrap_or(Value::Undefined.into())
            .to_object()?;

        let type_error = get_type_error(
            error_prototype.clone().into(),
            error_constructor.clone(),
            realm,
        )?;
        let range_error = get_range_error(
            error_prototype.clone().into(),
            error_constructor.clone(),
            realm,
        )?;
        let reference_error = get_reference_error(
            error_prototype.clone().into(),
            error_constructor.clone(),
            realm,
        )?;
        let syntax_error = get_syntax_error(
            error_prototype.clone().into(),
            error_constructor.clone(),
            realm,
        )?;

        let eval_error = get_eval_error(
            error_prototype.clone().into(),
            error_constructor.clone(),
            realm,
        )?;

        let uri_error = get_uri_error(
            error_prototype.clone().into(),
            error_constructor.clone(),
            realm,
        )?;

        let aggregate_error =
            get_aggregate_error(error_prototype.clone().into(), error_constructor, realm)?;

        let arraybuffer = ArrayBuffer::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let sharedarraybuffer = SharedArrayBuffer::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let data_view = DataView::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let typed_array = TypedArray::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let int8array = Int8Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let uint8array = Uint8Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let uint8clampedarray = Uint8ClampedArray::initialize_proto(
            Object::raw_with_proto(typed_array.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let int16array = Int16Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let uint16array = Uint16Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let int32array = Int32Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let uint32array = Uint32Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let float16array = Float16Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let float32array = Float32Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let float64array = Float64Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let bigint64array = BigInt64Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let biguint64array = BigUint64Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let atomics = Atomics::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let map = Map::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let weak_map = WeakMap::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let set = Set::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let weak_set = WeakSet::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let weak_ref = WeakRef::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let date = Date::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let reflect = Reflect::new(
            obj_prototype.clone().into(),
            func_prototype.clone().into(),
            realm,
        )?;

        let (temporal, temporal_protos) =
            get_temporal(obj_prototype.clone(), func_prototype.clone(), realm)?;

        let (intl, intl_protos) =
            crate::builtins::intl::get_intl(obj_prototype.clone(), func_prototype.clone(), realm)?;

        let (signal, signal_protos) = crate::builtins::signal::get_signal(
            obj_prototype.clone(),
            func_prototype.clone(),
            realm,
        )?;

        let promise = Promise::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let arguments = Arguments::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;

        let proxy = Proxy::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone()),
            func_prototype.clone().into(),
            realm,
        )?;
        
        let throw_type_error = get_throw_type_error(realm)?;

        Ok(Self {
            obj: obj_prototype,
            func: func_prototype,
            array: array_prototype,
            array_iter: array_iter_prototype,
            error: error_prototype,
            math: math_obj,
            string: string_prototype,
            number: number_prototype,
            boolean: boolean_prototype,
            symbol: symbol_prototype,
            bigint: bigint_prototype,
            regexp: regex,
            json,
            type_error,
            range_error,
            reference_error,
            syntax_error,
            eval_error,
            uri_error,
            aggregate_error,
            eval: None,
            arraybuffer,
            sharedarraybuffer,
            data_view,
            typed_array,
            int8array,
            uint8array,
            uint8clampedarray,
            int16array,
            uint16array,
            int32array,
            uint32array,
            float16array,
            float32array,
            float64array,
            bigint64array,
            biguint64array,
            atomics,
            map,
            weak_map,
            set,
            weak_set,
            weak_ref,
            date,
            reflect,
            temporal,
            temporal_duration: temporal_protos.duration,
            temporal_instant: temporal_protos.instant,
            temporal_now: temporal_protos.now,
            temporal_plain_date: temporal_protos.plain_date,
            temporal_plain_time: temporal_protos.plain_time,
            temporal_plain_date_time: temporal_protos.plain_date_time,
            temporal_plain_month_day: temporal_protos.plain_month_day,
            temporal_plain_year_month: temporal_protos.plain_year_month,
            temporal_zoned_date_time: temporal_protos.zoned_date_time,
            promise,
            generator_function: Object::null(),
            generator: Object::null(),
            async_generator_function: Object::null(),
            async_generator: Object::null(),
            signal,
            signal_state: signal_protos.state,
            signal_computed: signal_protos.computed,
            arguments,
            proxy,

            intl,
            intl_collator: intl_protos.collator,
            intl_date_time_format: intl_protos.date_time_format,
            intl_display_names: intl_protos.display_names,
            intl_duration_format: intl_protos.duration_format,
            intl_list_format: intl_protos.list_format,
            intl_locale: intl_protos.locale,
            intl_number_format: intl_protos.number_format,
            intl_plural_rules: intl_protos.plural_rules,
            intl_relative_time_format: intl_protos.relative_time_format,
            intl_segmenter: intl_protos.segmenter,
            
            throw_type_error,

            other: FxHashMap::default(),
        })
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
            array: Object::null(),
            array_iter: Object::null(),
            error: Object::null(),
            math: Object::null(),
            string: Object::null(),
            number: Object::null(),
            boolean: Object::null(),
            symbol: Object::null(),
            bigint: Object::null(),
            regexp: Object::null(),
            json: Object::null(),
            type_error: Object::null(),
            range_error: Object::null(),
            reference_error: Object::null(),
            syntax_error: Object::null(),
            eval_error: Object::null(),
            uri_error: Object::null(),
            aggregate_error: Object::null(),
            eval: None,
            arraybuffer: Object::null(),
            sharedarraybuffer: Object::null(),
            data_view: Object::null(),
            typed_array: Object::null(),
            int8array: Object::null(),
            uint8array: Object::null(),
            uint8clampedarray: Object::null(),
            int16array: Object::null(),
            uint16array: Object::null(),
            int32array: Object::null(),
            uint32array: Object::null(),
            float16array: Object::null(),
            float32array: Object::null(),
            float64array: Object::null(),
            bigint64array: Object::null(),
            biguint64array: Object::null(),
            atomics: Object::null(),
            map: Object::null(),
            weak_map: Object::null(),
            set: Object::null(),
            weak_set: Object::null(),
            weak_ref: Object::null(),
            date: Object::null(),
            reflect: Object::null(),
            temporal: Object::null(),
            temporal_duration: Object::null(),
            temporal_instant: Object::null(),
            temporal_now: Object::null(),
            temporal_plain_date: Object::null(),
            temporal_plain_time: Object::null(),
            temporal_plain_date_time: Object::null(),
            temporal_plain_month_day: Object::null(),
            temporal_plain_year_month: Object::null(),
            temporal_zoned_date_time: Object::null(),
            promise: Object::null(),
            generator_function: Object::null(),
            generator: Object::null(),
            async_generator_function: Object::null(),
            async_generator: Object::null(),
            signal: Object::null(),
            signal_state: Object::null(),
            signal_computed: Object::null(),
            arguments: Object::null(),
            proxy: Object::null(),
            intl: Object::null(),
            intl_collator: Object::null(),
            intl_date_time_format: Object::null(),
            intl_display_names: Object::null(),
            intl_duration_format: Object::null(),
            intl_list_format: Object::null(),
            intl_locale: Object::null(),
            intl_number_format: Object::null(),
            intl_plural_rules: Object::null(),
            intl_relative_time_format: Object::null(),
            intl_segmenter: Object::null(),
            throw_type_error: Object::null(),
            other: FxHashMap::default(),
        }
    }
}
