use crate::data::DataSection;
use crate::Instruction;
use yavashark_macro::object;

#[derive(Debug)]
#[object]
pub struct BytecodeFunction {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub body: Vec<Instruction>,
    pub data: DataSection,
}
