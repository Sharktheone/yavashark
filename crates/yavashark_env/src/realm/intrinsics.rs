use crate::array::{Array, ArrayIterator};
use crate::builtins::dataview::DataView;
use crate::builtins::{
    get_eval_error, get_range_error, get_reference_error, get_syntax_error, get_temporal,
    get_type_error, get_uri_error, ArrayBuffer, BigIntObj, BooleanObj, Date, Map, Math, NumberObj,
    Reflect, RegExp, StringObj, SymbolObj, JSON,
};
use crate::error::ErrorObj;
use crate::{Error, FunctionPrototype, Object, ObjectHandle, Prototype, Value, Variable};
use crate::builtins::bigint64array::BigInt64Array;
use crate::builtins::biguint64array::BigUint64Array;
use crate::builtins::float16array::Float16Array;
use crate::builtins::float32array::Float32Array;
use crate::builtins::float64array::Float64Array;
use crate::builtins::int16array::Int16Array;
use crate::builtins::int32array::Int32Array;
use crate::builtins::int8array::Int8Array;
use crate::builtins::uint16array::Uint16Array;
use crate::builtins::uint32array::Uint32Array;
use crate::builtins::uint8clampedarray::Uint8ClampedArray;
use crate::builtins::unit8array::Uint8Array;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Intrinsics {
    pub obj: ObjectHandle,
    pub func: ObjectHandle,
    pub(crate) array: ObjectHandle,
    pub(crate) array_iter: ObjectHandle,
    pub(crate) error: ObjectHandle,
    pub(crate) math: ObjectHandle,
    pub(crate) string: ObjectHandle,
    pub(crate) number: ObjectHandle,
    pub(crate) boolean: ObjectHandle,
    pub(crate) symbol: ObjectHandle,
    pub(crate) bigint: ObjectHandle,
    pub(crate) regexp: ObjectHandle,
    pub(crate) json: ObjectHandle,
    pub(crate) type_error: ObjectHandle,
    pub(crate) range_error: ObjectHandle,
    pub(crate) reference_error: ObjectHandle,
    pub(crate) syntax_error: ObjectHandle,
    pub(crate) eval_error: ObjectHandle,
    pub(crate) uri_error: ObjectHandle,
    pub(crate) eval: Option<ObjectHandle>,
    pub(crate) arraybuffer: ObjectHandle,
    pub(crate) data_view: ObjectHandle,
    pub(crate) typed_array: ObjectHandle,
    pub(crate) int8array: ObjectHandle,
    pub(crate) uint8array: ObjectHandle,
    pub(crate) uint8clampedarray: ObjectHandle,
    pub(crate) int16array: ObjectHandle,
    pub(crate) uint16array: ObjectHandle,
    pub(crate) int32array: ObjectHandle,
    pub(crate) uint32array: ObjectHandle,
    pub(crate) float16array: ObjectHandle,
    pub(crate) float32array: ObjectHandle,
    pub(crate) float64array: ObjectHandle,
    pub(crate) bigint64array: ObjectHandle,
    pub(crate) biguint64array: ObjectHandle,
    pub(crate) map: ObjectHandle,
    pub(crate) set: ObjectHandle,
    pub(crate) date: ObjectHandle,
    pub(crate) reflect: ObjectHandle,
    pub(crate) temporal: ObjectHandle,
    pub(crate) temporal_duration: ObjectHandle,
    pub(crate) temporal_instant: ObjectHandle,
    pub(crate) temporal_now: ObjectHandle,
    pub(crate) temporal_plain_date: ObjectHandle,
    pub(crate) temporal_plain_time: ObjectHandle,
    pub(crate) temporal_plain_date_time: ObjectHandle,
    pub(crate) temporal_plain_month_day: ObjectHandle,
    pub(crate) temporal_plain_year_month: ObjectHandle,
    pub(crate) temporal_zoned_date_time: ObjectHandle,
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
    constructor!(set);
    constructor!(date);

    obj!(json);
    obj!(math);
    obj!(reflect);
    obj!(temporal);
}

impl Intrinsics {
    pub(crate) fn new() -> Result<Self, Error> {
        let obj_prototype = ObjectHandle::new(Prototype::new());

        let func_prototype =
            ObjectHandle::new(FunctionPrototype::new(obj_prototype.clone().into()));

        {
            let obj_this = obj_prototype.clone().into();
            let obj = obj_prototype.get();

            let obj = obj.as_any();

            let proto = obj
                .downcast_ref::<Prototype>()
                .ok_or_else(|| Error::new("downcast_mut::<Prototype> failed"))?;

            proto.initialize(func_prototype.clone().into(), obj_this)?;
        }

        {
            let func = func_prototype.get();

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

        let type_error = get_type_error(
            error_prototype.clone().into(),
            func_prototype.clone().into(),
        )?;
        let range_error = get_range_error(
            error_prototype.clone().into(),
            func_prototype.clone().into(),
        )?;
        let reference_error = get_reference_error(
            error_prototype.clone().into(),
            func_prototype.clone().into(),
        )?;
        let syntax_error = get_syntax_error(
            error_prototype.clone().into(),
            func_prototype.clone().into(),
        )?;

        let eval_error = get_eval_error(
            error_prototype.clone().into(),
            func_prototype.clone().into(),
        )?;

        let uri_error = get_uri_error(
            error_prototype.clone().into(),
            func_prototype.clone().into(),
        )?;

        let arraybuffer = ArrayBuffer::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let data_view = DataView::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let typed_array = ArrayBuffer::initialize_proto(
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

        #[allow(clippy::similar_names)]
        let biguint64array = BigUint64Array::initialize_proto(
            Object::raw_with_proto(typed_array.clone().into()),
            func_prototype.clone().into(),
        )?;

        let map = Map::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )?;

        let set = Map::initialize_proto(
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
            set,
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
        })
    }
}
