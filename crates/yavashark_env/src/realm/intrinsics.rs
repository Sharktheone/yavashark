use yavashark_value::Obj;
use crate::array::{Array, ArrayIterator};
use crate::builtins::dataview::DataView;
use crate::builtins::{get_eval_error, get_range_error, get_reference_error, get_syntax_error, get_temporal, get_type_error, get_uri_error, ArrayBuffer, BigIntObj, BooleanObj, Date, Map, Math, NumberObj, Reflect, RegExp, StringObj, SymbolObj, JSON};
use crate::error::ErrorObj;
use crate::{Error, FunctionPrototype, Object, ObjectHandle, Prototype, Value, Variable};

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
    pub(crate) map: ObjectHandle,
    pub(crate) set: ObjectHandle,
    pub(crate) date: ObjectHandle,
    pub(crate) reflect: ObjectHandle,
    pub(crate) temporal: ObjectHandle,
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
        
        let temporal = get_temporal(obj_prototype.clone().into(), func_prototype.clone().into())?;
        

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
            map,
            set,
            date,
            reflect,
            temporal,
        })
    }
}
