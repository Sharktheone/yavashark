use crate::data::DataSection;
use crate::{FunctionBlueprint, Instruction};
use yavashark_env::{Context, Object, ObjectHandle};
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
    pub fn from_blueprint(b: FunctionBlueprint, ctx: &Context) -> ObjectHandle {
        let object = Object::raw(ctx);

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
