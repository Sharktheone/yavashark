use std::cell::RefMut;
use anyhow::anyhow;
use yavashark_value::Obj;
use crate::object::{Object, Prototype, array::Array};
use crate::{FunctionPrototype, ObjectHandle};
use crate::context::Context;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Prototypes {
    pub(crate) obj_prototype: ObjectHandle,
    pub(crate) func_prototype: ObjectHandle,
    pub(crate) array_prototype: ObjectHandle,
}


impl Prototypes {
    pub(crate) fn new() -> Result<Self, anyhow::Error> {
        let obj_prototype: Box<dyn Obj<Context>> = Box::new(Prototype::new());

        let obj_prototype = ObjectHandle::new(obj_prototype);

        let func_prototype: Box<dyn Obj<Context>> =
            Box::new(FunctionPrototype::new(&obj_prototype.clone().into()));
        let func_prototype = ObjectHandle::new(func_prototype);
        
        
        {
            let mut obj: RefMut<Box<dyn Obj<Context>>> = obj_prototype.get_mut().map_err(|e| anyhow!(format!("{e:?}")))?;
            let obj = obj.as_any_mut();

            let proto = obj
                .downcast_mut::<Prototype>()
                .ok_or_else(|| anyhow!("downcast_mut::<Prototype> failed"))?;

            proto.initialize(func_prototype.clone().into());
        }
        
        
        let array_prototype = Array::initialize_proto(Object::raw_with_proto(obj_prototype.clone().into()), func_prototype.clone().into())
            .map_err(|e| anyhow!(format!("{e:?}")))?;


        Ok(Self {
            obj_prototype,
            func_prototype,
            array_prototype,
        })
    }
    
    
}