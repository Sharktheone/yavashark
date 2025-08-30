use crate::data::VarName;
use crate::JmpAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlBlock {
    Try(TryBlock),
    DestructureArray(DestructureArray),
    DestructureObject(DestructureObject),
    Pat(PatBlock),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct TryBlock {
    pub exit: JmpAddr,
    pub catch: Option<JmpAddr>,
    pub catch_pat: Option<PatBlock>,
    pub finally: Option<JmpAddr>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PatBlock(VarName);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestructureArray {
    elems: Vec<Option<PatBlock>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestructureObject {
    props: Vec<(VarName, Option<PatBlock>)>,
}
