use slotmap::{DefaultKey, SlotMap};

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
}


pub struct FuncData {}

pub struct ExprData {}

pub struct StatData {}

pub struct VarDeclData {}