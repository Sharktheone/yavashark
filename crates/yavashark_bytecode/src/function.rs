use crate::data::DataSection;
use crate::{FunctionBlueprint, Instruction};
use yavashark_env::{Object, ObjectHandle};
use yavashark_env::realm::Realm;
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
        let object = Object::raw(realm);

        let this = Self {
            object,
            name: b.name,
            params: b.params,
            body: b.body,
            data: b.data,
        };

        ObjectHandle::new(this)
    }
}
