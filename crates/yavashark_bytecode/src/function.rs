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
