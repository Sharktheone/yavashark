use crate::{Error, ObjectHandle, Value};

pub trait NativeObject {
    fn initialize_proto(obj: &Value, func: &Value) -> Result<ObjectHandle, Error>;
}

pub struct Initializer {
    obj: Value,
    func: Value,
}

impl Initializer {
    pub fn new(obj: impl Into<Value>, func: impl Into<Value>) -> Self {
        let obj = obj.into();
        let func = func.into();

        Self { obj, func }
    }

    pub fn initialize_proto<T: NativeObject>(&self) -> Result<ObjectHandle, Error> {
        T::initialize_proto(&self.obj, &self.func)
    }
}
