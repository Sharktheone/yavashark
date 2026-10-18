use crate::block::Block;
use crate::types::Type;

pub struct FuncData {
    pub name: String,
    pub params: Vec<ArgType>,
    pub ret_type: Type,
    pub body: Block,
}

pub struct ArgType {
    pub name: String,
    pub ty: Type,
}
