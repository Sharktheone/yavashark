use crate::data::VarName;
use crate::JmpOffset;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlBlock {
    Try(TryBlock),
    Pat(PatBlock),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TryBlock {
    pub exit: JmpOffset,
    pub catch: Option<JmpOffset>,
    pub catch_pat: Option<PatBlock>,
    pub finally: Option<JmpOffset>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatBlock(VarName);