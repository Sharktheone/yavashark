use slotmap::{DefaultKey, SlotMap};
use crate::function::FuncData;

type NodeID = DefaultKey;

pub struct AST {
    nodes: SlotMap<NodeID, Node>,
    root: NodeID,
}


pub struct Node {
    children: Vec<NodeID>,
    data: NodeData,
}


pub enum NodeData {
    Func(FuncData),
    Expr(ExprData),
    Stmt(StatData),
    VarDecl(VarDeclData),
    Enum(EnumData),
    
}



pub struct ExprData {}

pub struct StatData {}

pub struct VarDeclData {}

pub struct EnumData {}