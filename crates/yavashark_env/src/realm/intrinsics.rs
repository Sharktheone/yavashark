use crate::array::{Array, ArrayIterator};
use crate::builtins::bigint64array::BigInt64Array;
use crate::builtins::biguint64array::BigUint64Array;
use crate::builtins::dataview::DataView;
use crate::builtins::float16array::Float16Array;
use crate::builtins::float32array::Float32Array;
use crate::builtins::float64array::Float64Array;
use crate::builtins::int16array::Int16Array;
use crate::builtins::int32array::Int32Array;
use crate::builtins::int8array::Int8Array;
use crate::builtins::typed_array::TypedArray;
use crate::builtins::uint16array::Uint16Array;
use crate::builtins::uint32array::Uint32Array;
use crate::builtins::uint8clampedarray::Uint8ClampedArray;
use crate::builtins::unit8array::Uint8Array;
use crate::builtins::{get_aggregate_error, get_eval_error, get_range_error, get_reference_error, get_syntax_error, get_temporal, get_type_error, get_uri_error, Arguments, ArrayBuffer, BigIntObj, BooleanObj, Date, Map, Math, NumberObj, Promise, Proxy, Reflect, RegExp, Set, StringObj, SymbolObj, WeakMap, WeakRef, WeakSet, JSON};
use crate::error::ErrorObj;
use crate::{Error, FunctionPrototype, Object, ObjectHandle, Prototype, Res, Value, Variable};
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

    pub other: FxHashMap<TypeId, ObjectHandle>,
}

macro_rules! constructor {
    ($name:ident) => {
        paste::paste! {
            pub fn [<$name _constructor>] (&self) -> Variable {
                self.$name
                    .get_property(&"constructor".into())
                    .unwrap_or(Value::Undefined.into())
                    .value //TODO: theoretically someone could use getters and setters here and then this would be wrong
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
    constructor!(date);
    constructor!(promise);
    constructor!(proxy);

    obj!(json);
    obj!(math);
    obj!(reflect);
    obj!(temporal);
    obj!(signal);
}

#[allow(clippy::similar_names)]
impl Intrinsics {
    pub(crate) fn new() -> Result<Self, Error> {
        let obj_prototype = ObjectHandle::new(Prototype::new());

        let func_prototype =
            ObjectHandle::new(FunctionPrototype::new(obj_prototype.clone().into()));

        {
            let obj_this = obj_prototype.clone().into();
            let obj = obj_prototype.guard();

            let obj = obj.as_any();

            let proto = obj
                .downcast_ref::<Prototype>()
                .ok_or_else(|| Error::new("downcast_mut::<Prototype> failed"))?;

            proto.initialize(func_prototype.clone().into(), obj_this)?;
        }

        {
            let func = func_prototype.guard();

            let func = func.as_any();

            let proto = func
                .downcast_ref::<FunctionPrototype>()
                .ok_or_else(|| Error::new("downcast_mut::<FunctionPrototype> failed"))?;

            proto.initialize(func_prototype.clone().into())?;
        }

        let array_prototype = Array::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let array_iter_prototype = ArrayIterator::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let error_prototype = ErrorObj::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let math_obj = Math::new(obj_prototype.clone(), func_prototype.clone())?;

        let string_prototype = StringObj::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let number_prototype = NumberObj::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let boolean_prototype = BooleanObj::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let symbol_prototype = SymbolObj::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let bigint_prototype = BigIntObj::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let regex = RegExp::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let json = JSON::new(obj_prototype.clone(), func_prototype.clone())?;

        let error_constructor = error_prototype
            .get_property(&"constructor".into())
            .unwrap_or(Value::Undefined.into())
            .value;

        let type_error = get_type_error(error_prototype.clone().into(), error_constructor.clone())?;
        let range_error =
            get_range_error(error_prototype.clone().into(), error_constructor.clone())?;
        let reference_error =
            get_reference_error(error_prototype.clone().into(), error_constructor.clone())?;
        let syntax_error =
            get_syntax_error(error_prototype.clone().into(), error_constructor.clone())?;

        let eval_error = get_eval_error(error_prototype.clone().into(), error_constructor.clone())?;

        let uri_error = get_uri_error(error_prototype.clone().into(), error_constructor.clone())?;

        let aggregate_error =
            get_aggregate_error(error_prototype.clone().into(), error_constructor)?;

        let arraybuffer = ArrayBuffer::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let data_view = DataView::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let typed_array = TypedArray::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let int8array = Int8Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone().into()),
            func_prototype.clone().into(),
        )?;

        let uint8array = Uint8Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone().into()),
            func_prototype.clone().into(),
        )?;

        let uint8clampedarray = Uint8ClampedArray::initialize_proto(
            Object::raw_with_proto(typed_array.clone().into()),
            func_prototype.clone().into(),
        )?;

        let int16array = Int16Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone().into()),
            func_prototype.clone().into(),
        )?;

        let uint16array = Uint16Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone().into()),
            func_prototype.clone().into(),
        )?;

        let int32array = Int32Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone().into()),
            func_prototype.clone().into(),
        )?;

        let uint32array = Uint32Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone().into()),
            func_prototype.clone().into(),
        )?;

        let float16array = Float16Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone().into()),
            func_prototype.clone().into(),
        )?;

        let float32array = Float32Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone().into()),
            func_prototype.clone().into(),
        )?;

        let float64array = Float64Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone().into()),
            func_prototype.clone().into(),
        )?;

        let bigint64array = BigInt64Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone().into()),
            func_prototype.clone().into(),
        )?;

        let biguint64array = BigUint64Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone().into()),
            func_prototype.clone().into(),
        )?;

        let map = Map::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let weak_map = WeakMap::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let set = Set::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let weak_set = WeakSet::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;
        
        let weak_ref = WeakRef::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let date = Date::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let reflect = Reflect::new(obj_prototype.clone().into(), func_prototype.clone().into())?;

        let (temporal, temporal_protos) =
            get_temporal(obj_prototype.clone(), func_prototype.clone())?;

        let (signal, signal_protos) =
            crate::builtins::signal::get_signal(obj_prototype.clone(), func_prototype.clone())?;

        let promise = Promise::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let arguments = Arguments::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let proxy = Proxy::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

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
