use crate::data::DataSection;
use crate::{FunctionBlueprint, Instruction};
use std::cell::RefCell;
use yavashark_env::realm::Realm;
use yavashark_env::{MutObject, ObjectHandle};
use yavashark_macro::object;

#[derive(Debug)]
#[object]
pub struct BytecodeFunction {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub body: Vec<Instruction>,
    pub data: DataSection,
}

impl BytecodeFunction {
    #[must_use]
    pub fn from_blueprint(b: FunctionBlueprint, realm: &Realm) -> ObjectHandle {
        let object = MutObject::with_proto(realm.intrinsics.func.clone().into());

        let this = Self {
            inner: RefCell::new(MutableBytecodeFunction { object }),
            name: b.name,
            params: b.params,
            body: b.body,
            data: b.data,
        };

        ObjectHandle::new(this)
    }
}
