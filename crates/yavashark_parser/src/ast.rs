use crate::function::FuncData;
use slotmap::{DefaultKey, SlotMap};

type NodeID = DefaultKey;

pub struct AST {
    pub nodes: SlotMap<NodeID, Node>,
    pub root: NodeID,
}

pub struct Node {
    pub children: Vec<NodeID>,
    pub data: NodeData,
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
