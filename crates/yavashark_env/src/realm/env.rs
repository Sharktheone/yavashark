use crate::scope::Module;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    pub modules: HashMap<PathBuf, Module>,
}
