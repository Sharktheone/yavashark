use crate::JmpOffset;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum ControlBlock {
    Try(TryBlock),
    Pat(PatBlock),
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct TryBlock {
    catch: Option<JmpOffset>,
    finally: Option<JmpOffset>,
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct PatBlock;