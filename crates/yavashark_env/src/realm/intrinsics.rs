use crate::array::{Array, ArrayIterator};
use crate::builtins::Math;
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
}

macro_rules! constructor {
    ($name:ident) => {
        paste::paste! {
            pub fn [<$name _constructor>] (&self) -> Variable {
                self.$name
                    .get_property(&"constructor".into())
                    .unwrap_or(Value::Undefined)
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
    obj!(math);
}

impl Intrinsics {
    pub(crate) fn new() -> Result<Self, Error> {
        let obj_prototype = ObjectHandle::new(Prototype::new());

        let func_prototype =
            ObjectHandle::new(FunctionPrototype::new(obj_prototype.clone().into()));

        {
            let obj_this = obj_prototype.clone().into();
            let mut obj = obj_prototype.get_mut()?;

            let obj = obj.as_any_mut();

            let proto = obj
                .downcast_mut::<Prototype>()
                .ok_or_else(|| Error::new("downcast_mut::<Prototype> failed"))?;

            proto.initialize(func_prototype.clone().into(), obj_this)?;
        }

        {
            let mut func = func_prototype.get_mut()?;

            let func = func.as_any_mut();

            let proto = func
                .downcast_mut::<FunctionPrototype>()
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

        let math_obj = Math::new(obj_prototype.clone().into(), func_prototype.clone().into())?;

        Ok(Self {
            obj: obj_prototype,
            func: func_prototype,
            array: array_prototype,
            array_iter: array_iter_prototype,
            error: error_prototype,
            math: math_obj,
        })
    }
}
