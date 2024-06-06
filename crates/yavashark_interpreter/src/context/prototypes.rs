use std::cell::RefMut;

use anyhow::anyhow;

use yavashark_value::Obj;

use crate::context::Context;
use crate::object::array::ArrayIterator;
use crate::object::{array::Array, Object, Prototype};
use crate::{FunctionPrototype, ObjectHandle, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prototypes {
    pub(crate) obj: ObjectHandle,
    pub(crate) func: ObjectHandle,
    pub(crate) array: ObjectHandle,
    pub(crate) array_iter: ObjectHandle,
}

impl Prototypes {
    pub(crate) fn new() -> Result<Self, anyhow::Error> {
        let obj_prototype = ObjectHandle::new(Prototype::new());

        let func_prototype =
            ObjectHandle::new(FunctionPrototype::new(&obj_prototype.clone().into()));

        {
            let mut obj =
                obj_prototype
                    .get_mut()
                    .map_err(|e| anyhow!(format!("{e:?}")))?;

            let obj = obj.as_any_mut();

            let proto = obj
                .downcast_mut::<Prototype>()
                .ok_or_else(|| anyhow!("downcast_mut::<Prototype> failed"))?;

            proto.initialize(func_prototype.clone().into());
        }

        {
            let mut func = 
                func_prototype
                    .get_mut()
                    .map_err(|e| anyhow!(format!("{e:?}")))?;

            let func = func.as_any_mut();

            let proto = func
                .downcast_mut::<FunctionPrototype>()
                .ok_or_else(|| anyhow!("downcast_mut::<FunctionPrototype> failed"))?;

            proto.initialize(func_prototype.clone().into());
        }

        let array_prototype = Array::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )
        .map_err(|e| anyhow!(format!("{e:?}")))?;
        
        let array_iter_prototype = ArrayIterator::initialize_proto(
            Object::raw_with_proto(obj_prototype.clone().into()),
            func_prototype.clone().into(),
        )
        .map_err(|e| anyhow!(format!("{e:?}")))?;

        
        Ok(Self {
            obj: obj_prototype,
            func: func_prototype,
            array: array_prototype,
            array_iter: array_iter_prototype,
        })
    }
}
