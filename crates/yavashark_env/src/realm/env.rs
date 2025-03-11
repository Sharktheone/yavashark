use std::collections::HashMap;
use std::path::PathBuf;
use crate::scope::Module;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    pub modules: HashMap<PathBuf, Module>,
}
