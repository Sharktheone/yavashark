use crate::scope::Module;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Environment {
    pub modules: HashMap<PathBuf, Module>,
}

impl Clone for Environment {
    fn clone(&self) -> Self {
        Self {
            modules: self.modules.clone(),
        }
    }
}

impl PartialEq for Environment {
    fn eq(&self, other: &Self) -> bool {
        self.modules == other.modules
    }
}

impl Eq for Environment {}
